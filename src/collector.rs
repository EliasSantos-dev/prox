use crate::model::{PortEntry, Process, Protocol};
use std::collections::HashMap;

/// Socket em escuta lido do /proc, antes de associar ao processo.
pub struct RawSocket {
    pub port: u16,
    pub protocol: Protocol,
    pub inode: u64,
}

/// Combina sockets em escuta com o mapa inode->processo numa lista final.
/// Função pura: testável sem tocar no /proc.
pub fn build_entries(
    sockets: Vec<RawSocket>,
    inode_to_process: &HashMap<u64, Process>,
) -> Vec<PortEntry> {
    sockets
        .into_iter()
        .map(|s| PortEntry {
            port: s.port,
            protocol: s.protocol,
            process: inode_to_process.get(&s.inode).cloned(),
        })
        .collect()
}

use std::io;

pub fn snapshot() -> io::Result<Vec<PortEntry>> {
    let mut sockets: Vec<RawSocket> = Vec::new();
    if let Ok(tcp) = procfs::net::tcp() {
        for e in tcp {
            if e.state == procfs::net::TcpState::Listen {
                sockets.push(RawSocket {
                    port: e.local_address.port(),
                    protocol: Protocol::Tcp,
                    inode: e.inode,
                });
            }
        }
    }
    if let Ok(tcp6) = procfs::net::tcp6() {
        for e in tcp6 {
            if e.state == procfs::net::TcpState::Listen {
                sockets.push(RawSocket {
                    port: e.local_address.port(),
                    protocol: Protocol::Tcp,
                    inode: e.inode,
                });
            }
        }
    }
    if let Ok(udp) = procfs::net::udp() {
        for e in udp {
            sockets.push(RawSocket {
                port: e.local_address.port(),
                protocol: Protocol::Udp,
                inode: e.inode,
            });
        }
    }
    if let Ok(udp6) = procfs::net::udp6() {
        for e in udp6 {
            sockets.push(RawSocket {
                port: e.local_address.port(),
                protocol: Protocol::Udp,
                inode: e.inode,
            });
        }
    }
    let inode_to_process = build_inode_map();
    let mut entries = build_entries(sockets, &inode_to_process);
    crate::model::sort(&mut entries, crate::model::SortKey::Port);
    Ok(entries)
}

fn build_inode_map() -> HashMap<u64, Process> {
    let mut map = HashMap::new();
    let Ok(procs) = procfs::process::all_processes() else {
        return map;
    };
    for p in procs.flatten() {
        let Ok(stat) = p.stat() else { continue };
        let name = stat.comm.clone();
        let cmdline = p.cmdline().ok().map(|v| v.join(" ")).unwrap_or_default();
        let user = p
            .uid()
            .ok()
            .and_then(uzers_name)
            .unwrap_or_else(|| "?".into());
        let process = Process {
            pid: p.pid(),
            name,
            cmdline,
            user,
        };
        if let Ok(fds) = p.fd() {
            for fd in fds.flatten() {
                if let procfs::process::FDTarget::Socket(inode) = fd.target {
                    map.entry(inode).or_insert_with(|| process.clone());
                }
            }
        }
    }
    map
}

fn uzers_name(uid: u32) -> Option<String> {
    let content = std::fs::read_to_string("/etc/passwd").ok()?;
    for line in content.lines() {
        let mut f = line.split(':');
        let nm = f.next()?;
        let _passwd = f.next()?;
        let line_uid: u32 = f.next()?.parse().ok()?;
        if line_uid == uid {
            return Some(nm.to_string());
        }
    }
    Some(uid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn associates_socket_to_process_by_inode() {
        let sockets = vec![
            RawSocket {
                port: 3000,
                protocol: Protocol::Tcp,
                inode: 111,
            },
            RawSocket {
                port: 9999,
                protocol: Protocol::Tcp,
                inode: 222,
            },
        ];
        let mut map = HashMap::new();
        map.insert(
            111u64,
            Process {
                pid: 42,
                name: "node".into(),
                cmdline: "npm".into(),
                user: "elias".into(),
            },
        );

        let out = build_entries(sockets, &map);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].process.as_ref().unwrap().pid, 42);
        assert!(out[1].process.is_none());
    }
}

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

    // Read /etc/passwd once and build uid -> username lookup table.
    let passwd_map: HashMap<u32, String> = std::fs::read_to_string("/etc/passwd")
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut f = line.split(':');
            let nm = f.next()?.to_string();
            let _passwd = f.next()?;
            let uid: u32 = f.next()?.parse().ok()?;
            Some((uid, nm))
        })
        .collect();

    let Ok(procs) = procfs::process::all_processes() else {
        return map;
    };
    for p in procs.flatten() {
        // First, collect socket inodes for this process.
        // Only pay the cost of stat/cmdline/uid resolution if there are any.
        let Ok(fds) = p.fd() else { continue };
        let socket_inodes: Vec<u64> = fds
            .flatten()
            .filter_map(|fd| {
                if let procfs::process::FDTarget::Socket(inode) = fd.target {
                    Some(inode)
                } else {
                    None
                }
            })
            .collect();

        if socket_inodes.is_empty() {
            continue;
        }

        // This process owns at least one socket — now resolve its metadata.
        let Ok(stat) = p.stat() else { continue };
        let name = stat.comm.clone();
        let cmdline = p.cmdline().ok().map(|v| v.join(" ")).unwrap_or_default();
        let user = p
            .uid()
            .ok()
            .map(|uid| uid_to_username(uid, &passwd_map))
            .unwrap_or_else(|| "?".into());
        let process = Process {
            pid: p.pid(),
            name,
            cmdline,
            user,
        };

        // First process to claim a given inode wins.
        for inode in socket_inodes {
            map.entry(inode).or_insert_with(|| process.clone());
        }
    }
    map
}

fn uid_to_username(uid: u32, passwd_map: &HashMap<u32, String>) -> String {
    passwd_map
        .get(&uid)
        .cloned()
        .unwrap_or_else(|| uid.to_string())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn label(&self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    pub pid: i32,
    pub name: String,
    pub cmdline: String,
    pub user: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortEntry {
    pub port: u16,
    pub protocol: Protocol,
    pub process: Option<Process>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Port,
    Name,
    Pid,
}

/// Filtra entradas cujo termo aparece na porta, nome do processo, PID ou usuário.
/// Comparação case-insensitive. Termo vazio devolve tudo.
pub fn filter(entries: &[PortEntry], term: &str) -> Vec<PortEntry> {
    if term.is_empty() {
        return entries.to_vec();
    }
    let needle = term.to_lowercase();
    entries
        .iter()
        .filter(|e| {
            let mut hay = format!("{} {}", e.port, e.protocol.label().to_lowercase());
            if let Some(p) = &e.process {
                hay.push_str(&format!(" {} {} {} {}", p.pid, p.name, p.cmdline, p.user));
            }
            hay.to_lowercase().contains(&needle)
        })
        .cloned()
        .collect()
}

/// Ordena in-place. Entradas sem processo vão para o fim quando a chave
/// depende do processo (Name/Pid).
pub fn sort(entries: &mut [PortEntry], key: SortKey) {
    match key {
        SortKey::Port => entries.sort_by_key(|e| e.port),
        SortKey::Name => entries.sort_by(|a, b| {
            let an = a.process.as_ref().map(|p| p.name.as_str());
            let bn = b.process.as_ref().map(|p| p.name.as_str());
            an.is_none().cmp(&bn.is_none()).then(an.cmp(&bn))
        }),
        SortKey::Pid => entries.sort_by(|a, b| {
            let ap = a.process.as_ref().map(|p| p.pid);
            let bp = b.process.as_ref().map(|p| p.pid);
            ap.is_none().cmp(&bp.is_none()).then(ap.cmp(&bp))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PortEntry {
        PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            process: Some(Process {
                pid: 42,
                name: "node".into(),
                cmdline: "npm run dev".into(),
                user: "elias".into(),
            }),
        }
    }

    #[test]
    fn protocol_label_is_uppercase() {
        assert_eq!(Protocol::Tcp.label(), "TCP");
        assert_eq!(Protocol::Udp.label(), "UDP");
    }

    #[test]
    fn entry_holds_process() {
        let e = sample();
        assert_eq!(e.port, 3000);
        assert_eq!(e.process.unwrap().name, "node");
    }

    fn dataset() -> Vec<PortEntry> {
        vec![
            PortEntry { port: 5432, protocol: Protocol::Tcp, process: Some(Process { pid: 10, name: "postgres".into(), cmdline: "/usr/bin/postgres".into(), user: "postgres".into() }) },
            PortEntry { port: 3000, protocol: Protocol::Tcp, process: Some(Process { pid: 42, name: "node".into(), cmdline: "npm run dev".into(), user: "elias".into() }) },
            PortEntry { port: 6379, protocol: Protocol::Tcp, process: None },
        ]
    }

    #[test]
    fn filter_matches_process_name() {
        let out = filter(&dataset(), "node");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].port, 3000);
    }

    #[test]
    fn filter_matches_port_number() {
        let out = filter(&dataset(), "5432");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].process.as_ref().unwrap().name, "postgres");
    }

    #[test]
    fn filter_is_case_insensitive() {
        assert_eq!(filter(&dataset(), "NODE").len(), 1);
    }

    #[test]
    fn empty_filter_returns_all() {
        assert_eq!(filter(&dataset(), "").len(), 3);
    }

    #[test]
    fn sort_by_port_ascending() {
        let mut d = dataset();
        sort(&mut d, SortKey::Port);
        let ports: Vec<u16> = d.iter().map(|e| e.port).collect();
        assert_eq!(ports, vec![3000, 5432, 6379]);
    }

    #[test]
    fn sort_by_pid_puts_none_last() {
        let mut d = dataset();
        sort(&mut d, SortKey::Pid);
        assert!(d.last().unwrap().process.is_none());
    }
}

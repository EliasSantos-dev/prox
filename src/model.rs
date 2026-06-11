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
}

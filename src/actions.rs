use nix::sys::signal::{kill as nix_kill, Signal};
use nix::unistd::Pid;

#[derive(Debug, PartialEq, Eq)]
pub enum KillOutcome {
    Terminated,
    PermissionDenied,
    NotFound,
    Error(String),
}

/// Envia SIGTERM ao processo. Mapeia erros comuns para variantes claras.
pub fn term(pid: i32) -> KillOutcome {
    match nix_kill(Pid::from_raw(pid), Signal::SIGTERM) {
        Ok(()) => KillOutcome::Terminated,
        Err(nix::errno::Errno::EPERM) => KillOutcome::PermissionDenied,
        Err(nix::errno::Errno::ESRCH) => KillOutcome::NotFound,
        Err(e) => KillOutcome::Error(e.to_string()),
    }
}

/// Força SIGKILL (usado se o processo sobreviver ao SIGTERM).
pub fn force(pid: i32) -> KillOutcome {
    match nix_kill(Pid::from_raw(pid), Signal::SIGKILL) {
        Ok(()) => KillOutcome::Terminated,
        Err(nix::errno::Errno::EPERM) => KillOutcome::PermissionDenied,
        Err(nix::errno::Errno::ESRCH) => KillOutcome::NotFound,
        Err(e) => KillOutcome::Error(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn killing_nonexistent_pid_is_not_found() {
        let outcome = term(2_000_000_000);
        assert_eq!(outcome, KillOutcome::NotFound);
    }
}

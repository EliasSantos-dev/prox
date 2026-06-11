use crate::model::{filter, sort, PortEntry, SortKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filtering,
    ConfirmKill(i32),
}

/// Eventos de alto nível que o app entende (desacoplado do crossterm).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Down,
    Up,
    StartFilter,
    FilterChar(char),
    FilterBackspace,
    ClearFilter,
    ToggleSort,
    RequestKill,
    ConfirmYes,
    ConfirmNo,
    Refreshed(Vec<PortEntry>),
    Quit,
}

pub struct App {
    all: Vec<PortEntry>,
    pub visible: Vec<PortEntry>,
    pub selected: usize,
    pub filter_text: String,
    pub mode: Mode,
    pub sort_key: SortKey,
    pub status: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(entries: Vec<PortEntry>) -> Self {
        let mut app = App {
            all: entries,
            visible: Vec::new(),
            selected: 0,
            filter_text: String::new(),
            mode: Mode::Normal,
            sort_key: SortKey::Port,
            status: None,
            should_quit: false,
        };
        app.recompute();
        app
    }

    fn recompute(&mut self) {
        let mut v = filter(&self.all, &self.filter_text);
        sort(&mut v, self.sort_key);
        self.visible = v;
        if self.selected >= self.visible.len() {
            self.selected = self.visible.len().saturating_sub(1);
        }
    }

    pub fn selected_entry(&self) -> Option<&PortEntry> {
        self.visible.get(self.selected)
    }

    pub fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Quit => self.should_quit = true,
            Event::Down => {
                if self.selected + 1 < self.visible.len() {
                    self.selected += 1;
                }
            }
            Event::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            Event::StartFilter => self.mode = Mode::Filtering,
            Event::FilterChar(c) => {
                self.filter_text.push(c);
                self.recompute();
            }
            Event::FilterBackspace => {
                self.filter_text.pop();
                self.recompute();
            }
            Event::ClearFilter => {
                self.filter_text.clear();
                self.mode = Mode::Normal;
                self.recompute();
            }
            Event::ToggleSort => {
                self.sort_key = match self.sort_key {
                    SortKey::Port => SortKey::Name,
                    SortKey::Name => SortKey::Pid,
                    SortKey::Pid => SortKey::Port,
                };
                self.recompute();
            }
            Event::RequestKill => {
                if let Some(p) = self.selected_entry().and_then(|e| e.process.as_ref()) {
                    self.mode = Mode::ConfirmKill(p.pid);
                } else {
                    self.status = Some("Sem processo associado para matar".into());
                }
            }
            Event::ConfirmNo => {
                self.mode = Mode::Normal;
            }
            Event::ConfirmYes => {
                self.mode = Mode::Normal;
            }
            Event::Refreshed(entries) => {
                self.all = entries;
                self.recompute();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Process, Protocol};

    fn entry(port: u16, name: &str, pid: i32) -> PortEntry {
        PortEntry {
            port,
            protocol: Protocol::Tcp,
            process: Some(Process {
                pid,
                name: name.into(),
                cmdline: "".into(),
                user: "elias".into(),
            }),
        }
    }

    fn app() -> App {
        App::new(vec![entry(3000, "node", 42), entry(5432, "postgres", 10)])
    }

    #[test]
    fn down_and_up_move_selection_within_bounds() {
        let mut a = app();
        assert_eq!(a.selected, 0);
        a.on_event(Event::Down);
        assert_eq!(a.selected, 1);
        a.on_event(Event::Down);
        assert_eq!(a.selected, 1);
        a.on_event(Event::Up);
        assert_eq!(a.selected, 0);
        a.on_event(Event::Up);
        assert_eq!(a.selected, 0);
    }

    #[test]
    fn typing_filter_narrows_visible() {
        let mut a = app();
        a.on_event(Event::StartFilter);
        assert_eq!(a.mode, Mode::Filtering);
        for c in "node".chars() {
            a.on_event(Event::FilterChar(c));
        }
        assert_eq!(a.visible.len(), 1);
        assert_eq!(a.visible[0].port, 3000);
    }

    #[test]
    fn clear_filter_restores_all_and_normal_mode() {
        let mut a = app();
        a.on_event(Event::StartFilter);
        a.on_event(Event::FilterChar('x'));
        assert_eq!(a.visible.len(), 0);
        a.on_event(Event::ClearFilter);
        assert_eq!(a.visible.len(), 2);
        assert_eq!(a.mode, Mode::Normal);
    }

    #[test]
    fn request_kill_enters_confirm_with_pid() {
        let mut a = app();
        a.on_event(Event::RequestKill);
        assert_eq!(a.mode, Mode::ConfirmKill(42));
    }

    #[test]
    fn confirm_no_cancels_kill() {
        let mut a = app();
        a.on_event(Event::RequestKill);
        a.on_event(Event::ConfirmNo);
        assert_eq!(a.mode, Mode::Normal);
    }

    #[test]
    fn request_kill_without_process_sets_status_and_stays_normal() {
        let mut a = App::new(vec![PortEntry {
            port: 9999,
            protocol: Protocol::Tcp,
            process: None,
        }]);
        a.on_event(Event::RequestKill);
        assert_eq!(a.mode, Mode::Normal);
        assert!(a.status.is_some());
    }

    #[test]
    fn refresh_with_shorter_list_clamps_selection() {
        let mut a = app();
        a.on_event(Event::Down); // selected = 1
        assert_eq!(a.selected, 1);
        a.on_event(Event::Refreshed(vec![entry(3000, "node", 42)])); // only 1 entry now
        assert!(a.selected < a.visible.len().max(1));
        assert_eq!(a.selected, 0);
    }

    #[test]
    fn toggle_sort_cycles_keys() {
        let mut a = app();
        assert_eq!(a.sort_key, SortKey::Port);
        a.on_event(Event::ToggleSort);
        assert_eq!(a.sort_key, SortKey::Name);
        a.on_event(Event::ToggleSort);
        assert_eq!(a.sort_key, SortKey::Pid);
        a.on_event(Event::ToggleSort);
        assert_eq!(a.sort_key, SortKey::Port);
    }
}

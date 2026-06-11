mod actions;
mod app;
mod collector;
mod model;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use app::{App, Event, Mode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

const REFRESH: Duration = Duration::from_millis(1000);
const TICK: Duration = Duration::from_millis(100);

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let initial = collector::snapshot().unwrap_or_default();
    let mut app = App::new(initial);
    let mut last_refresh = Instant::now();

    while !app.should_quit {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(TICK)? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Special-case: 'r' in Normal mode does immediate refresh
                    if app.mode == Mode::Normal && key.code == KeyCode::Char('r') {
                        if let Ok(entries) = collector::snapshot() {
                            app.on_event(Event::Refreshed(entries));
                        }
                        last_refresh = Instant::now();
                        continue;
                    }
                    if let Some(ev) = map_key(&app.mode, key.code) {
                        let confirm = ev == Event::ConfirmYes;
                        app.on_event(ev);
                        if confirm {
                            do_kill(&mut app);
                        }
                    }
                }
            }
        }

        if last_refresh.elapsed() >= REFRESH {
            if let Ok(entries) = collector::snapshot() {
                app.on_event(Event::Refreshed(entries));
            }
            last_refresh = Instant::now();
        }
    }
    Ok(())
}

fn map_key(mode: &Mode, code: KeyCode) -> Option<Event> {
    match mode {
        Mode::ConfirmKill => match code {
            KeyCode::Char('s') | KeyCode::Char('y') => Some(Event::ConfirmYes),
            KeyCode::Char('n') | KeyCode::Esc => Some(Event::ConfirmNo),
            _ => None,
        },
        Mode::Filtering => match code {
            KeyCode::Esc => Some(Event::ClearFilter),
            KeyCode::Backspace => Some(Event::FilterBackspace),
            KeyCode::Enter => Some(Event::StartFilter),
            KeyCode::Char(c) => Some(Event::FilterChar(c)),
            _ => None,
        },
        Mode::Normal => match code {
            KeyCode::Char('q') => Some(Event::Quit),
            KeyCode::Down | KeyCode::Char('j') => Some(Event::Down),
            KeyCode::Up | KeyCode::Char('k') => Some(Event::Up),
            KeyCode::Char('/') => Some(Event::StartFilter),
            KeyCode::Char('s') => Some(Event::ToggleSort),
            KeyCode::Char('K') | KeyCode::Delete => Some(Event::RequestKill),
            _ => None,
        },
    }
}

fn do_kill(app: &mut App) {
    let Some(pid) = app.pending_kill.take() else {
        return;
    };
    use actions::KillOutcome;
    let outcome = actions::term(pid);
    app.status = Some(match outcome {
        KillOutcome::Terminated => {
            std::thread::sleep(Duration::from_millis(300));
            if procfs::process::Process::new(pid).is_ok() {
                match actions::force(pid) {
                    KillOutcome::Terminated => format!("PID {pid} morto (SIGKILL)"),
                    other => format!("Falha ao forçar PID {pid}: {other:?}"),
                }
            } else {
                format!("PID {pid} encerrado")
            }
        }
        KillOutcome::PermissionDenied => format!("Sem permissão para matar PID {pid}"),
        KillOutcome::NotFound => format!("PID {pid} não existe mais"),
        KillOutcome::Error(e) => format!("Erro ao matar PID {pid}: {e}"),
    });
}

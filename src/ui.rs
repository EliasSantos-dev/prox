use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    let header = Row::new(["PORTA", "PROTO", "PID", "PROCESSO", "USUÁRIO", "CMD"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = app.visible.iter().enumerate().map(|(i, e)| {
        let (pid, name, user, cmd) = match &e.process {
            Some(p) => (
                p.pid.to_string(),
                p.name.clone(),
                p.user.clone(),
                p.cmdline.clone(),
            ),
            None => ("-".into(), "-".into(), "-".into(), "-".into()),
        };
        let style = if i == app.selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(e.port.to_string()),
            Cell::from(e.protocol.label()),
            Cell::from(pid),
            Cell::from(name),
            Cell::from(user),
            Cell::from(cmd),
        ])
        .style(style)
    });

    let widths = [
        Constraint::Length(7),
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Length(16),
        Constraint::Length(10),
        Constraint::Min(10),
    ];
    let title = format!(" prox — {} portas ", app.visible.len());
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, chunks[0]);

    let filter_line = match app.mode {
        Mode::Filtering => format!("/{}", app.filter_text),
        _ if !app.filter_text.is_empty() => format!("filtro: {}", app.filter_text),
        _ => String::new(),
    };
    f.render_widget(Paragraph::new(filter_line), chunks[1]);

    let help = match app.mode {
        Mode::ConfirmKill => {
            let pid = app.pending_kill.unwrap_or(0);
            format!("Matar PID {pid}? (s/n)")
        }
        _ => app.status.clone().unwrap_or_else(|| {
            "↑↓ mover  / filtrar  K matar  s ordenar  r recarregar  q sair".into()
        }),
    };
    f.render_widget(Paragraph::new(Line::from(help)), chunks[2]);
}

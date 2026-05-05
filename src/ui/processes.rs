use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Row, Table};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let sorted = app.sorted_processes();

    let header_labels = [
        "PID",
        "Name",
        "CPU%",
        "MEM%",
        "User",
        "Command",
    ];

    let header_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    let widths = [
        Constraint::Length(8),
        Constraint::Length(18),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Min(20),
    ];

    let max_rows = (area.height.saturating_sub(3)) as usize;

    let start = app.process_selected.saturating_sub(max_rows / 2);
    let end = (start + max_rows).min(sorted.len());
    let window = sorted[start..end].iter();

    let rows: Vec<Row> = window
        .enumerate()
        .map(|(i, p)| {
            let cpu_color = if p.cpu_usage > 50.0 {
                Color::Red
            } else if p.cpu_usage > 10.0 {
                Color::Yellow
            } else {
                Color::White
            };

            let mem_str = format!("{:.1}", p.mem_percent);
            let cpu_str = format!("{:.1}", p.cpu_usage);

            let style = if i == app.process_selected.saturating_sub(start) {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                cpu_str,
                mem_str,
                p.user.clone(),
                p.command.clone(),
            ]).style(style).style(Style::default().fg(cpu_color))
        })
        .collect();

    let header = Row::new(header_labels)
        .style(header_style)
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(format!(
                    " Processes ({}) | Sort: {} {} ",
                    app.total_processes,
                    app.sort_column.header(),
                    if app.sort_descending { "▼" } else { "▲" }
                ))
                .border_style(Style::new().fg(Color::White)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌ ");

    frame.render_widget(table, area);
}

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Gauge, Paragraph, Row, Table};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if app.disks.is_empty() {
        draw_placeholder(frame, area, "No disks found");
        return;
    }

    let main_chunks = Layout::vertical([
        Constraint::Percentage(70),
        Constraint::Percentage(30),
    ])
    .split(area);

    draw_disk_table(frame, main_chunks[0], app);
    draw_disk_gauges(frame, main_chunks[1], app);
}

fn draw_disk_table(frame: &mut Frame, area: Rect, app: &App) {
    let widths = [
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Min(10),
    ];

    let header = Row::new([
        "Mount",
        "Name",
        "FS",
        "Total",
        "Used",
        "Use%",
        "Removable",
    ])
    .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .disks
        .iter()
        .map(|d| {
            let used_pct = d.usage_percent() * 100.0;
            let pct_color = if used_pct < 50.0 {
                Color::Green
            } else if used_pct < 80.0 {
                Color::Yellow
            } else {
                Color::Red
            };

            let removable = if d.is_removable { "Yes" } else { "No" };

            Row::new(vec![
                d.mount_point.clone(),
                d.name.clone(),
                d.file_system.clone(),
                format_bytes(d.total),
                format_bytes(d.used()),
                format!("{used_pct:.0}%"),
                removable.to_string(),
            ]).style(Style::default().fg(pct_color))
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(format!(" Disks ({}) ", app.disks.len()))
                .border_style(Style::new().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn draw_disk_gauges(frame: &mut Frame, area: Rect, app: &App) {
    let num_disks = app.disks.len();
    let constraints: Vec<Constraint> = if num_disks <= 1 {
        vec![Constraint::Percentage(100)]
    } else {
        vec![Constraint::Percentage(50); num_disks]
    };

    let chunks = Layout::vertical(&constraints).split(area);

    for (i, disk) in app.disks.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }

        let ratio = disk.usage_percent();
        let color = if ratio < 0.5 {
            Color::Green
        } else if ratio < 0.8 {
            Color::Yellow
        } else {
            Color::Red
        };

        let label = format!(
            "{}: {}/{} ({:.0}%)",
            truncate(&disk.mount_point, 20),
            format_bytes(disk.used()),
            format_bytes(disk.total),
            ratio * 100.0,
        );

        let gauge = Gauge::default()
            .block(
                Block::bordered()
                    .title(format!(" {} ", truncate(&disk.name, 20)))
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .gauge_style(Style::new().fg(color))
            .ratio(ratio)
            .label(label);

        frame.render_widget(gauge, chunks[i]);
    }
}

fn draw_placeholder(frame: &mut Frame, area: Rect, text: &str) {
    let block = Block::bordered()
        .title(" Disks ")
        .border_style(Style::new().fg(Color::Cyan));

    let widget = Paragraph::new(text).block(block);
    frame.render_widget(widget, area);
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_099_511_627_776 {
        format!("{:.2} TiB", bytes as f64 / 1_099_511_627_776.0)
    } else if bytes >= 1_073_741_824 {
        format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1} KiB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max.saturating_sub(1)])
    } else {
        s.to_string()
    }
}

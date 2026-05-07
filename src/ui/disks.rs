use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, LineGauge, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.disks.is_empty() {
        draw_placeholder(frame, area, "No disks found");
        return;
    }

    let main_chunks = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    draw_disk_table(frame, main_chunks[0], app);
    draw_disk_gauges(frame, main_chunks[1], app);
}

fn draw_disk_table(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Length(1),
    ]).split(area);

    let table_area = chunks[0];
    let scrollbar_area = chunks[1];

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
        )
        .row_highlight_style(Style::new().bg(Color::Yellow).fg(Color::Black))
        .highlight_symbol("│> ");

    let mut state = TableState::default();
    state.select(Some(app.disk_selected));

    frame.render_stateful_widget(table, table_area, &mut state);

    let mut scrollbar_state = ScrollbarState::new(app.disks.len())
        .position(app.disk_selected)
        .viewport_content_length(table_area.height.saturating_sub(3) as usize);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    frame.render_stateful_widget(
        scrollbar,
        scrollbar_area,
        &mut scrollbar_state,
    );
}

fn draw_disk_gauges(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::bordered()
        .title(" Disk Usage ")
        .border_style(Style::new().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.disks.is_empty() {
        return;
    }

    // Split into gauges area and scrollbar area
    let chunks = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Length(1),
    ]).split(inner);

    let gauges_area = chunks[0];
    let scrollbar_area = chunks[1];

    // Each gauge takes 1 line
    let visible_count = gauges_area.height as usize;
    let total_disks = app.disks.len();

    // Bound the scroll position
    let max_scroll = total_disks.saturating_sub(visible_count);
    if app.disk_gauges_scroll > max_scroll {
        app.disk_gauges_scroll = max_scroll;
    }
    let start = app.disk_gauges_scroll;
    let end = (start + visible_count).min(total_disks);

    if start < end {
        let constraints: Vec<Constraint> = vec![Constraint::Length(1); end - start];
        let gauge_chunks = Layout::vertical(&constraints).split(gauges_area);

        for (i, disk) in app.disks[start..end].iter().enumerate() {
            let ratio = disk.usage_percent();
            let color = if ratio < 0.5 {
                Color::Green
            } else if ratio < 0.8 {
                Color::Yellow
            } else {
                Color::Red
            };

            let label = format!(
                "{} ({}): {}/{} ({:.0}%)",
                truncate(&disk.name, 12),
                truncate(&disk.mount_point, 15),
                format_bytes(disk.used()),
                format_bytes(disk.total),
                ratio * 100.0,
            );

            let gauge = LineGauge::default()
                .filled_style(Style::new().fg(color))
                .unfilled_style(Style::new().fg(Color::DarkGray))
                .ratio(ratio)
                .label(label);

            frame.render_widget(gauge, gauge_chunks[i]);
        }
    }

    // Draw scrollbar if needed
    if total_disks > visible_count {
        let mut scrollbar_state = ScrollbarState::new(total_disks)
            .position(start)
            .viewport_content_length(visible_count);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        frame.render_stateful_widget(
            scrollbar,
            scrollbar_area,
            &mut scrollbar_state,
        );
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

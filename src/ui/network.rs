use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::widgets::{Block, Paragraph, Row, Sparkline, Table};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Percentage(50),
        Constraint::Min(1),
    ])
    .split(area);

    draw_total_throughput(frame, chunks[0], app);
    draw_sparklines(frame, chunks[1], app);
    draw_interface_table(frame, chunks[2], app);
    draw_interface_details(frame, chunks[3], app);
}

fn draw_total_throughput(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let rx_str = format_speed(app.total_network_rx);
    let tx_str = format_speed(app.total_network_tx);

    let rx_widget = Paragraph::new(format!("RX: {rx_str}"))
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .block(
            Block::bordered()
                .title(" Download ")
                .border_style(Style::new().fg(Color::Green)),
        );

    let tx_widget = Paragraph::new(format!("TX: {tx_str}"))
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .block(
            Block::bordered()
                .title(" Upload ")
                .border_style(Style::new().fg(Color::Blue)),
        );

    frame.render_widget(rx_widget, chunks[0]);
    frame.render_widget(tx_widget, chunks[1]);
}

fn draw_sparklines(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let rx_data: Vec<u64> = app.net_rx_history.clone();
    let tx_data: Vec<u64> = app.net_tx_history.clone();

    let max_rx = *rx_data.iter().max().unwrap_or(&1);
    let max_tx = *tx_data.iter().max().unwrap_or(&1);

    let rx_sparkline = Sparkline::default()
        .block(
            Block::bordered()
                .title(" RX History (60s) ")
                .border_style(Style::new().fg(Color::Green)),
        )
        .data(&rx_data)
        .max(max_rx)
        .direction(ratatui::widgets::RenderDirection::RightToLeft)
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS);

    let tx_sparkline = Sparkline::default()
        .block(
            Block::bordered()
                .title(" TX History (60s) ")
                .border_style(Style::new().fg(Color::Blue)),
        )
        .data(&tx_data)
        .max(max_tx)
        .direction(ratatui::widgets::RenderDirection::RightToLeft)
        .style(Style::default().fg(Color::Blue))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(rx_sparkline, chunks[0]);
    frame.render_widget(tx_sparkline, chunks[1]);
}

fn draw_interface_table(frame: &mut Frame, area: Rect, app: &App) {
    if app.network_interfaces.is_empty() {
        draw_placeholder(frame, area, "No network interfaces found");
        return;
    }

    let widths = [
        Constraint::Length(12),
        Constraint::Length(18),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(12),
    ];

    let header = Row::new(["Interface", "MAC", "IPv4", "Total RX", "Total TX", "RX/s"])
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .network_interfaces
        .iter()
        .map(|iface| {
            Row::new(vec![
                iface.name.clone(),
                iface.mac_address.clone(),
                iface.ipv4.clone(),
                format_bytes(iface.rx_bytes),
                format_bytes(iface.tx_bytes),
                format_speed(iface.rx_speed),
            ])
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(format!(" Interfaces ({}) ", app.network_interfaces.len()))
                .border_style(Style::new().fg(Color::Cyan)),
        );

    frame.render_widget(table, area);
}

fn draw_interface_details(frame: &mut Frame, area: Rect, app: &App) {
    if app.network_interfaces.is_empty() {
        return;
    }

    let lines: Vec<String> = app
        .network_interfaces
        .iter()
        .map(|iface| {
            format!(
                "  {} | {} | RX: {} | TX: {} | Speed: {}/{}",
                iface.name,
                iface.ipv4,
                format_bytes(iface.rx_bytes),
                format_bytes(iface.tx_bytes),
                format_speed(iface.rx_speed),
                format_speed(iface.tx_speed),
            )
        })
        .collect();

    let widget = Paragraph::new(lines.join("\n"))
        .block(
            Block::bordered()
                .title(" Interface Summary ")
                .border_style(Style::new().fg(Color::Cyan)),
        );

    frame.render_widget(widget, area);
}

fn draw_placeholder(frame: &mut Frame, area: Rect, text: &str) {
    let block = Block::bordered()
        .title(" Network ")
        .border_style(Style::new().fg(Color::Cyan));

    let widget = Paragraph::new(text).block(block);
    frame.render_widget(widget, area);
}

fn format_speed(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB/s", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB/s", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.2} KB/s", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B/s")
    }
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

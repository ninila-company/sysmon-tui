use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Gauge, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut crate::app::App) {
    let has_swap = app.has_swap();

    let constraints = if has_swap {
        vec![
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ]
    } else {
        vec![
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ]
    };

    let chunks = Layout::vertical(&constraints).split(area);

    draw_ram_gauge(frame, chunks[0], app);

    if has_swap {
        draw_swap_gauge(frame, chunks[1], app);
        draw_memory_details(frame, chunks[2], app);
    } else {
        draw_memory_details(frame, chunks[1], app);
    }
}

fn draw_ram_gauge(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let ratio = app.mem_usage_percent();
    let color = usage_color(ratio);

    let label = format!(
        "{:.1} / {:.1} GB ({:.1}%)",
        bytes_to_gib(app.used_memory),
        bytes_to_gib(app.total_memory),
        ratio * 100.0
    );

    let gauge = Gauge::default()
        .block(
            Block::bordered()
                .title(" RAM ")
                .border_style(Style::new().fg(Color::Yellow)),
        )
        .gauge_style(Style::new().fg(color))
        .ratio(ratio)
        .label(label);

    frame.render_widget(gauge, area);
}

fn draw_swap_gauge(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let ratio = app.swap_usage_percent();
    let color = usage_color(ratio);

    let label = format!(
        "{:.1} / {:.1} GB ({:.1}%)",
        bytes_to_gib(app.used_swap),
        bytes_to_gib(app.total_swap),
        ratio * 100.0
    );

    let gauge = Gauge::default()
        .block(
            Block::bordered()
                .title(" Swap ")
                .border_style(Style::new().fg(Color::Magenta)),
        )
        .gauge_style(Style::new().fg(color))
        .ratio(ratio)
        .label(label);

    frame.render_widget(gauge, area);
}

fn draw_memory_details(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let used = app.used_memory;
    let available = app.available_memory;
    let total = app.total_memory;
    let cached = used.saturating_sub(total.saturating_sub(available)) / 2;
    let buffers = (total.saturating_sub(available)) / 4;

    let used_pct = app.mem_usage_percent() * 100.0;
    let avail_pct = if total > 0 {
        (available as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let mut lines = Vec::new();

    lines.push(format!("  Used:      {:>10}  ({:.1}%)", format_bytes(used), used_pct));
    lines.push(format!("  Available: {:>10}  ({:.1}%)", format_bytes(available), avail_pct));
    lines.push(format!("  Cached:    {:>10}", format_bytes(cached)));
    lines.push(format!("  Buffers:   {:>10}", format_bytes(buffers)));

    if app.has_swap() {
        lines.push(String::new());
        lines.push(format!("  Swap Used: {:>10}  ({:.1}%)", format_bytes(app.used_swap), app.swap_usage_percent() * 100.0));
        lines.push(format!("  Swap Free: {:>10}", format_bytes(app.total_swap.saturating_sub(app.used_swap))));
    }

    let widget = Paragraph::new(lines.join("\n"))
        .block(
            Block::bordered()
                .title(" Memory Details ")
                .border_style(Style::new().fg(Color::Yellow)),
        );

    frame.render_widget(widget, area);
}

fn usage_color(ratio: f64) -> Color {
    if ratio < 0.5 {
        Color::Green
    } else if ratio < 0.8 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.2} KiB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}

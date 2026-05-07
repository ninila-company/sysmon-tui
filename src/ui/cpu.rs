use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::{Block, Gauge, Paragraph, Sparkline};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut crate::app::App) {
    let main_chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ])
    .split(area);

    draw_left_panel(frame, main_chunks[0], app);
    draw_right_panel(frame, main_chunks[1], app);
}

fn draw_left_panel(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let chunks = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(1),
    ])
    .split(area);

    draw_cpu_gauge(frame, chunks[0], app);
    draw_per_core_bars(frame, chunks[1], app);
}

fn draw_cpu_gauge(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let cpu_usage = app.cpu_history.last().copied().unwrap_or(0.0);
    let ratio = (cpu_usage / 100.0).clamp(0.0, 1.0);

    let color = if cpu_usage < 50.0 {
        Color::Green
    } else if cpu_usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let label = format!("{:.1}% | {} logical cores", cpu_usage, app.cpu_cores.len());

    let gauge = Gauge::default()
        .block(Block::bordered().title(" Overall CPU ").border_style(Style::new().fg(Color::Green)))
        .gauge_style(Style::new().fg(color))
        .ratio(ratio as f64)
        .label(label);

    frame.render_widget(gauge, area);
}

fn draw_per_core_bars(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let block = Block::bordered()
        .title(" Per-Core ")
        .border_style(Style::new().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 1 || app.cpu_cores.is_empty() {
        return;
    }

    let num_cores = app.cpu_cores.len();
    let max_rows = inner.height as usize;
    let cols = if num_cores <= max_rows {
        1
    } else {
        2
    };

    let rows = (num_cores + cols - 1) / cols;

    let col_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);

    for row in 0..rows {
        let row_area = Rect {
            x: inner.x,
            y: inner.y + row as u16 * 2,
            width: inner.width,
            height: 2,
        };

        let [left_area, right_area] = col_layout.areas(row_area);

        let left_idx = row;
        let right_idx = row + rows;

        if left_idx < num_cores {
            render_core_bar(frame, left_area, left_idx, &app.cpu_cores);
        }

        if cols == 2 && right_idx < num_cores {
            render_core_bar(frame, right_area, right_idx, &app.cpu_cores);
        }
    }
}

fn render_core_bar(frame: &mut Frame, area: Rect, index: usize, cores: &[f32]) {
    if index >= cores.len() {
        return;
    }

    let usage = cores[index];
    let ratio = (usage / 100.0).clamp(0.0, 1.0);

    let color = if usage < 50.0 {
        Color::Green
    } else if usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let label = format!("Core {index:>2}: {usage:.1}%");

    let gauge = Gauge::default()
        .gauge_style(Style::new().fg(color))
        .ratio(ratio as f64)
        .label(label);

    frame.render_widget(gauge, area);
}

fn draw_right_panel(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
    ])
    .split(area);

    draw_sparkline(frame, chunks[0], app);
    draw_cpu_info(frame, chunks[1], app);
}

fn draw_sparkline(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let data: Vec<u64> = app
        .cpu_history
        .iter()
        .map(|v| (*v * 100.0) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(Block::bordered().title(" CPU History (60s) ").border_style(Style::new().fg(Color::Green)))
        .data(&data)
        .max(100)
        .direction(ratatui::widgets::RenderDirection::RightToLeft)
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS);

    frame.render_widget(sparkline, area);
}

fn draw_cpu_info(frame: &mut Frame, area: Rect, app: &crate::app::App) {
    let brand = &app.cpu_brand;
    let freq = if app.cpu_freq > 0.0 {
        format!("{:.2} GHz", app.cpu_freq / 1000.0)
    } else {
        "N/A".to_string()
    };

    let load_avg = if let Some(avg) = &app.load_average {
        format!("{:.2} {:.2} {:.2}", avg.one, avg.five, avg.fifteen)
    } else {
        "N/A".to_string()
    };

    let info_text = format!(
        "Model: {brand}\nFrequency: {freq}\nLoad Avg: {load_avg}",
    );

    let widget = Paragraph::new(info_text)
        .block(Block::bordered().title(" CPU Info ").border_style(Style::new().fg(Color::Green)));

    frame.render_widget(widget, area);
}

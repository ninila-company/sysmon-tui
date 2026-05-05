use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Paragraph};
use ratatui::text::{Line, Span};

use crate::app::{App, Tab};

mod cpu;
mod memory;
mod processes;
mod disks;
mod network;
mod docker;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app);
    draw_main_content(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(35),
        Constraint::Percentage(35),
        Constraint::Percentage(30),
    ])
    .split(area);

    let cpu_usage = app.cpu_history.last().copied().unwrap_or(0.0);
    let cpu_color = if cpu_usage < 50.0 {
        Color::Green
    } else if cpu_usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let primary_ip = app.network_interfaces.iter()
        .find(|i| i.ipv4 != "N/A")
        .map(|i| i.ipv4.as_str())
        .unwrap_or("N/A");

    let cpu_block = Block::bordered()
        .title(format!(" {}@{} | {} ", app.hostname, primary_ip, app.os_version))
        .border_style(Style::new().fg(Color::Green));

    let cpu_text = format!("{:.1}% | {} logical cores", cpu_usage, app.cpu_cores.len());
    let cpu_widget = Paragraph::new(cpu_text)
        .style(Style::new().fg(cpu_color).bold())
        .block(cpu_block);
    frame.render_widget(cpu_widget, chunks[0]);

    let mem_used = app.used_memory;
    let mem_ratio = app.mem_usage_percent();

    let mem_color = if mem_ratio < 0.5 {
        Color::Green
    } else if mem_ratio < 0.8 {
        Color::Yellow
    } else {
        Color::Red
    };

    let mem_block = Block::bordered()
        .title(" Memory ")
        .border_style(Style::new().fg(Color::Yellow));

    let mem_text = format!(
        "{:.1} / {:.1} GB",
        bytes_to_gb(mem_used),
        bytes_to_gb(app.total_memory)
    );
    let mem_widget = Paragraph::new(mem_text)
        .style(Style::new().fg(mem_color).bold())
        .block(mem_block);
    frame.render_widget(mem_widget, chunks[1]);

    let info_block = Block::bordered()
        .title(" sysmon-tui ")
        .border_style(Style::new().fg(Color::Cyan));

    let info_lines = vec![
        format!("v{} | {}ms", env!("CARGO_PKG_VERSION"), app.refresh_rate_ms),
    ];

    let info_widget = Paragraph::new(info_lines.join("\n"))
        .style(Style::new().fg(Color::Cyan))
        .block(info_block);
    frame.render_widget(info_widget, chunks[2]);
}

fn draw_main_content(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    match app.selected_tab {
        Tab::Cpu => {
            cpu::draw(frame, area, app);
        }
        Tab::Memory => {
            memory::draw(frame, area, app);
        }
        Tab::Processes => {
            processes::draw(frame, area, app);
        }
        Tab::Disks => {
            disks::draw(frame, area, app);
        }
        Tab::Network => {
            network::draw(frame, area, app);
        }
        Tab::Docker => {
            docker::draw(frame, area, app);
        }
    }
}

fn draw_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if app.show_help {
        let block = Block::bordered()
            .title(" Help ")
            .border_style(Style::new().fg(Color::White));

        let help_text = "1:CPU 2:MEM 3:PROC 4:DISK 5:NET 6:DOCKER | Tab:next | ↑↓:scroll | PgUp/PgDn | s:sort cycle | ?:help | q:quit";
        let widget = Paragraph::new(help_text).block(block);
        frame.render_widget(widget, area);
    } else {
        let footer_chunks = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(22),
        ])
        .split(area);

        let block = Block::bordered().border_style(Style::new().fg(Color::DarkGray));

        let tabs = ["1:CPU", "2:MEM", "3:PROC", "4:DISK", "5:NET", "6:DOCKER"];
        let active = app.selected_tab as usize;

        let spans: Vec<Span> = tabs
            .iter()
            .enumerate()
            .map(|(i, &tab)| {
                if i == active {
                    Span::styled(format!(" {tab} "), Style::new().bg(Color::Blue).fg(Color::White))
                } else {
                    Span::styled(format!(" {tab} "), Style::new().fg(Color::Gray))
                }
            })
            .collect();

        let mut line_spans = spans;
        line_spans.push(Span::raw(" | "));
        line_spans.push(Span::styled("↑↓:scroll", Style::new().fg(Color::Gray)));
        line_spans.push(Span::raw(" | "));
        line_spans.push(Span::styled("s:sort", Style::new().fg(Color::Gray)));
        line_spans.push(Span::raw(" | "));
        line_spans.push(Span::styled("?:help", Style::new().fg(Color::Gray)));
        line_spans.push(Span::raw(" | "));
        line_spans.push(Span::styled("q:quit", Style::new().fg(Color::Red)));

        let widget = Paragraph::new(Line::from(line_spans)).block(block);
        frame.render_widget(widget, footer_chunks[0]);

        let uptime_text = format!("Up: {}", app.formatted_uptime());
        let uptime_block = Block::bordered()
            .title(" Uptime ")
            .border_style(Style::new().fg(Color::DarkGray));
        let uptime_widget = Paragraph::new(uptime_text)
            .style(Style::new().fg(Color::Cyan))
            .block(uptime_block);
        frame.render_widget(uptime_widget, footer_chunks[1]);
    }
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Paragraph, Row, Table};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Some(ref err) = app.docker_error {
        draw_error(frame, area, err);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ])
    .split(area);

    draw_containers(frame, chunks[0], app);
    draw_images(frame, chunks[1], app);
}

fn draw_containers(frame: &mut Frame, area: Rect, app: &App) {
    let running = app.docker_containers.iter().filter(|c| c.state == "running").count();
    let total = app.docker_containers.len();

    let widths = [
        Constraint::Length(14),
        Constraint::Length(18),
        Constraint::Length(22),
        Constraint::Length(14),
        Constraint::Length(20),
        Constraint::Length(16),
        Constraint::Min(10),
    ];

    let header = Row::new(["ID", "Name", "Image", "State", "Created", "Status", "Ports"])
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .docker_containers
        .iter()
        .map(|c| {
            let state_color = match c.state.as_str() {
                "running" => Color::Green,
                "exited" | "dead" => Color::Red,
                "paused" => Color::Yellow,
                "created" => Color::Blue,
                _ => Color::White,
            };

            Row::new(vec![
                c.id.clone(),
                c.name.clone(),
                c.image.clone(),
                c.state.clone(),
                c.created.clone(),
                c.status.clone(),
                c.ports.clone(),
            ])
            .style(Style::default().fg(state_color))
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(format!(" Containers ({total}) [{running} running] "))
                .border_style(Style::new().fg(Color::Blue)),
        );

    frame.render_widget(table, area);
}

fn draw_images(frame: &mut Frame, area: Rect, app: &App) {
    let widths = [
        Constraint::Length(14),
        Constraint::Length(30),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Min(16),
    ];

    let header = Row::new(["ID", "Repository", "Tag", "Size", "Created"])
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .docker_images
        .iter()
        .map(|img| {
            Row::new(vec![
                img.id.clone(),
                img.repository.clone(),
                img.tag.clone(),
                img.size.clone(),
                img.created.clone(),
            ])
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(format!(" Images ({}) ", app.docker_images.len()))
                .border_style(Style::new().fg(Color::Magenta)),
        );

    frame.render_widget(table, area);
}

fn draw_error(frame: &mut Frame, area: Rect, error: &str) {
    let block = Block::bordered()
        .title(" Docker ")
        .border_style(Style::new().fg(Color::Red));

    let text = format!(
        "Could not connect to Docker daemon.\n\nError: {error}\n\nMake sure Docker is running and\nthe socket /var/run/docker.sock is accessible."
    );
    let widget = Paragraph::new(text).block(block);
    frame.render_widget(widget, area);
}

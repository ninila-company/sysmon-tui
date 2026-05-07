use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, Tab};

#[allow(dead_code)]
pub enum AppAction {
    Quit,
    NextTab,
    PrevTab,
    SelectTab(Tab),
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Left,
    Right,
    ToggleSort,
    ToggleHelp,
    None,
}

pub fn handle_event(_app: &mut App) -> anyhow::Result<AppAction> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind,
            ..
        }) = event::read()?
        {
            if kind != KeyEventKind::Press {
                return Ok(AppAction::None);
            }

            if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                return Ok(match code {
                    KeyCode::Char('q') | KeyCode::Esc => AppAction::Quit,
                    KeyCode::Tab => AppAction::NextTab,
                    KeyCode::Char('1') => AppAction::SelectTab(Tab::Cpu),
                    KeyCode::Char('2') => AppAction::SelectTab(Tab::Memory),
                    KeyCode::Char('3') => AppAction::SelectTab(Tab::Processes),
                    KeyCode::Char('4') => AppAction::SelectTab(Tab::Disks),
                    KeyCode::Char('5') => AppAction::SelectTab(Tab::Network),
                    KeyCode::Char('6') => AppAction::SelectTab(Tab::Docker),
                    KeyCode::Char('?') => AppAction::ToggleHelp,
                    KeyCode::Char('s') | KeyCode::Char('S') => AppAction::ToggleSort,
                    KeyCode::Up => AppAction::ScrollUp,
                    KeyCode::Down => AppAction::ScrollDown,
                    KeyCode::PageUp => AppAction::PageUp,
                    KeyCode::PageDown => AppAction::PageDown,
                    KeyCode::Left => AppAction::Left,
                    KeyCode::Right => AppAction::Right,
                    _ => AppAction::None,
                });
            }
        }
    }
    Ok(AppAction::None)
}

pub fn apply_action(app: &mut App, action: AppAction) {
    match action {
        AppAction::Quit => app.running = false,
        AppAction::NextTab => {
            app.selected_tab = app.selected_tab.next();
        }
        AppAction::PrevTab => {}
        AppAction::SelectTab(tab) => {
            app.selected_tab = tab;
        }
        AppAction::ScrollUp => {
            if app.selected_tab == Tab::Disks {
                app.disk_scroll_up();
            } else {
                app.scroll_up();
            }
        }
        AppAction::ScrollDown => {
            if app.selected_tab == Tab::Disks {
                app.disk_scroll_down();
            } else {
                app.scroll_down();
            }
        }
        AppAction::PageUp => {
            if app.selected_tab == Tab::Disks {
                app.disk_page_up();
            } else {
                app.page_up();
            }
        }
        AppAction::PageDown => {
            if app.selected_tab == Tab::Disks {
                app.disk_page_down();
            } else {
                app.page_down();
            }
        }
        AppAction::Left => {
            if app.selected_tab == Tab::Disks {
                app.disk_gauges_scroll_up();
            }
        }
        AppAction::Right => {
            if app.selected_tab == Tab::Disks {
                app.disk_gauges_scroll_down(app.visible_disk_gauges.max(1));
            }
        }
        AppAction::ToggleSort => {
            if app.selected_tab == Tab::Processes {
                if app.sort_descending {
                    app.sort_descending = false;
                } else {
                    app.sort_column = app.sort_column.next();
                    app.sort_descending = true;
                }
            }
        }
        AppAction::ToggleHelp => {
            app.show_help = !app.show_help;
        }
        AppAction::None => {}
    }
}

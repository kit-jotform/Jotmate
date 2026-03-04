use crossterm::event::{self, Event, KeyCode, KeyModifiers};

#[derive(Debug)]
pub enum AppEvent {
    OutputLine(String),
    CommandFinished(i32),
}

pub fn poll_crossterm_event() -> Option<Event> {
    if event::poll(std::time::Duration::from_millis(16)).unwrap_or(false) {
        event::read().ok()
    } else {
        None
    }
}

/// Returns true if the app should quit
pub fn handle_key(
    app: &mut crate::tui::app::App,
    event: Event,
) -> bool {
    use crate::tui::app::AppState;

    let Event::Key(key) = event else {
        return false;
    };

    match &app.state.clone() {
        AppState::Menu => match key.code {
            KeyCode::Up | KeyCode::Char('k') => app.move_menu_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_menu_down(),
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Enter => {
                // Selection is handled in the main loop by returning menu_selection
                // We signal via a side-channel: set state to indicate selection made
                // (the main loop will check menu_selection and dispatch)
                return false; // handled in main event loop
            }
            _ => {}
        },
        AppState::Running { finished, .. } => match key.code {
            KeyCode::Up | KeyCode::PageUp | KeyCode::Char('k') => app.scroll_up(3),
            KeyCode::Down | KeyCode::PageDown | KeyCode::Char('j') => app.scroll_down(3),
            KeyCode::Char('q') | KeyCode::Esc if *finished => {
                app.state = AppState::Menu;
                app.clear_output();
            }
            _ => {}
        },
        AppState::Settings => match key.code {
            KeyCode::Tab => app.settings.move_next(),
            KeyCode::BackTab => app.settings.move_prev(),
            KeyCode::Enter => app.settings.toggle_or_start_edit(),
            KeyCode::Esc => {
                app.settings.cancel_edit();
                if !app.settings.editing {
                    app.state = AppState::Menu;
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.settings.commit_edit();
                app.settings.apply_to_config(&mut app.config);
                let _ = crate::config::save(&app.config);
                app.state = AppState::Menu;
            }
            KeyCode::Char(c) if app.settings.editing => app.settings.handle_char(c),
            KeyCode::Backspace if app.settings.editing => app.settings.handle_backspace(),
            _ => {}
        },
    }

    false
}

pub fn handle_app_event(app: &mut crate::tui::app::App, event: AppEvent) {
    match event {
        AppEvent::OutputLine(line) => app.append_output_line(line),
        AppEvent::CommandFinished(code) => {
            let msg = if code == 0 {
                "\x1b[32m✓ Command completed successfully\x1b[0m".to_string()
            } else {
                format!("\x1b[31m✗ Command exited with code {code}\x1b[0m")
            };
            app.append_output_line(String::new());
            app.append_output_line(msg);
            app.append_output_line("Press q or Esc to return to menu".to_string());
            app.mark_command_finished();
        }
    }
}

pub fn is_enter_key(event: &Event) -> bool {
    matches!(event, Event::Key(k) if k.code == KeyCode::Enter)
}

pub mod app;
pub mod events;
pub mod streaming;
pub mod widgets;

use anyhow::Result;
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen},
    event::DisableMouseCapture,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::{self, Stdout};
use tokio::sync::mpsc;

use app::{ActiveCommand, App, AppState, MENU_ITEMS};
use events::{handle_app_event, handle_key, poll_crossterm_event, AppEvent};
use widgets::{
    logo::LogoWidget,
    menu::MenuWidget,
    output_pane::OutputPane,
    settings_form::SettingsFormWidget,
};

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub async fn run_interactive() -> Result<()> {
    let config = crate::config::load()?;
    let mut app = App::new(config);
    let mut terminal = setup_terminal()?;

    let result = event_loop(&mut terminal, &mut app).await;

    restore_terminal(&mut terminal)?;
    result
}

pub async fn run_settings() -> Result<()> {
    let config = crate::config::load()?;
    let mut app = App::new(config);
    app.state = AppState::Settings;
    let mut terminal = setup_terminal()?;

    let result = event_loop(&mut terminal, &mut app).await;

    restore_terminal(&mut terminal)?;
    result
}

async fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(256);

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            match &app.state {
                AppState::Settings => {
                    let widget = SettingsFormWidget { state: &app.settings };
                    frame.render_widget(widget, area);
                }
                _ => {
                    // Main layout: logo top, menu+output below
                    let logo_height = 9u16;
                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(logo_height),
                            Constraint::Min(0),
                        ])
                        .split(area);

                    frame.render_widget(LogoWidget, vertical[0]);

                    let horizontal = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(24),
                            Constraint::Min(0),
                        ])
                        .split(vertical[1]);

                    let is_running = matches!(&app.state, AppState::Running { .. });
                    let menu_active = !is_running;

                    MenuWidget {
                        selected: app.menu_selection,
                        active: menu_active,
                    }
                    .render_into(horizontal[0], frame.buffer_mut());

                    let (output_title, output_active) = match &app.state {
                        AppState::Running { command, .. } => {
                            let label = match command {
                                ActiveCommand::Sync => "Sync Output",
                                ActiveCommand::Time => "Time Output",
                            };
                            (label, true)
                        }
                        _ => ("Output", false),
                    };

                    frame.render_widget(
                        OutputPane {
                            lines: &app.output_lines,
                            scroll_offset: app.output_scroll,
                            title: output_title,
                            active: output_active,
                        },
                        horizontal[1],
                    );
                }
            }
        })?;

        // Drain app events (non-blocking)
        loop {
            match rx.try_recv() {
                Ok(event) => handle_app_event(app, event),
                Err(_) => break,
            }
        }

        // Poll keyboard
        if let Some(term_event) = poll_crossterm_event() {
            let is_enter = events::is_enter_key(&term_event);
            let should_quit = handle_key(app, term_event);

            if should_quit {
                break;
            }

            // Handle Enter in Menu state
            if is_enter {
                if let AppState::Menu = &app.state {
                    match app.menu_selection {
                        0 => {
                            // Sync
                            app.state = AppState::Running {
                                command: ActiveCommand::Sync,
                                finished: false,
                            };
                            app.clear_output();
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                let args = crate::cli::SyncArgs::default();
                                if let Err(e) = crate::sync::run_tui(args, tx_clone.clone()).await {
                                    let _ = tx_clone
                                        .send(AppEvent::OutputLine(format!("Error: {e}")))
                                        .await;
                                    let _ = tx_clone.send(AppEvent::CommandFinished(1)).await;
                                }
                            });
                        }
                        1 => {
                            // Time
                            app.state = AppState::Running {
                                command: ActiveCommand::Time,
                                finished: false,
                            };
                            app.clear_output();
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                let args = crate::cli::TimeArgs::default();
                                if let Err(e) = run_time_tui(args, tx_clone.clone()).await {
                                    let _ = tx_clone
                                        .send(AppEvent::OutputLine(format!("Error: {e}")))
                                        .await;
                                    let _ = tx_clone.send(AppEvent::CommandFinished(1)).await;
                                }
                            });
                        }
                        2 => {
                            // Settings
                            app.state = AppState::Settings;
                        }
                        3 => {
                            // Quit
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        if matches!(app.state, AppState::Quitting) {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }

    Ok(())
}

/// Run the time command and stream output as lines to AppEvent channel.
async fn run_time_tui(
    args: crate::cli::TimeArgs,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    // Run time::run() and capture output by redirecting prints
    // Since time::run() uses eprintln!/println!, we capture via a subprocess approach
    // or we can just run it directly and the TUI won't show live output.
    // For a clean live output, spawn self as subprocess.
    use std::process::Stdio;
    use tokio::process::Command;

    let current_exe = std::env::current_exe()?;
    let mut cmd_args = vec!["time".to_string()];
    if args.skip_current_week {
        cmd_args.push("--skip-current-week".to_string());
    }
    if args.no_cache {
        cmd_args.push("--no-cache".to_string());
    }

    let child = Command::new(&current_exe)
        .args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    streaming::stream_command_output(child, tx).await
}

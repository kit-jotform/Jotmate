pub mod app;
pub mod events;
pub mod streaming;
pub mod widgets;

use anyhow::Result;
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::DisableMouseCapture,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
    Terminal,
};
use std::io::{self, Stdout};
use tokio::sync::mpsc;

use app::{ActiveCommand, App, AppState};
use events::{handle_app_event, handle_key, poll_crossterm_event, AppEvent};
use widgets::{
    logo::{BigTextWidget, ChafaImageWidget, BIG_TEXT_HEIGHT},
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
    init_logo(&mut app);
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

const LOGO_PNG: &[u8] = include_bytes!("../../assets/logo2.png");

// Image render size in terminal cells. Height drives the logo row height.
const LOGO_COLS: u32 = 18;
const LOGO_ROWS: u32 = 10;

fn is_limited_terminal() -> bool {
    // macOS Terminal.app doesn't render chafa block graphics correctly
    matches!(
        std::env::var("TERM_PROGRAM").as_deref(),
        Ok("Apple_Terminal")
    )
}

fn init_logo(app: &mut App) {
    if is_limited_terminal() { return; }
    let Ok(img) = image::load_from_memory(LOGO_PNG) else { return };
    let rgba = img.to_rgba8();
    let ansi = render_logo_chafa(rgba.as_raw(), rgba.width(), rgba.height());
    app.logo_ansi = ansi.lines().map(|l| l.to_string()).collect();
}

fn render_logo_chafa(pixels: &[u8], pix_width: u32, pix_height: u32) -> String {
    use std::ffi::CString;
    // Use BLOCK + HALF only — excludes BORDER (box-drawing) chars that look like artifacts
    const TAG_BLOCK: i32 = 8;
    const TAG_HALF: i32 = 768; // HHALF | VHALF
    let tags = TAG_BLOCK | TAG_HALF;

    unsafe {
        let symbol_map = chafa_sys::chafa_symbol_map_new();
        chafa_sys::chafa_symbol_map_add_by_tags(symbol_map, tags);

        let config = chafa_sys::chafa_canvas_config_new();
        chafa_sys::chafa_canvas_config_set_geometry(config, LOGO_COLS as i32, LOGO_ROWS as i32);
        chafa_sys::chafa_canvas_config_set_symbol_map(config, symbol_map);

        let canvas = chafa_sys::chafa_canvas_new(config);
        chafa_sys::chafa_canvas_draw_all_pixels(
            canvas,
            chafa_sys::ChafaPixelType_CHAFA_PIXEL_RGBA8_UNASSOCIATED,
            pixels.as_ptr(),
            pix_width as i32,
            pix_height as i32,
            (pix_width * 4) as i32,
        );

        let gstring = chafa_sys::chafa_canvas_build_ansi(canvas);
        let ansistr = (*gstring).str_;
        let result = CString::from_raw(ansistr).to_string_lossy().to_string();

        chafa_sys::chafa_canvas_unref(canvas);
        chafa_sys::chafa_canvas_config_unref(config);
        chafa_sys::chafa_symbol_map_unref(symbol_map);

        result
    }
}

fn render_status_bar(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let branch = get_git_branch();
    let now = chrono::Local::now();
    let time_str = now.format("%H:%M").to_string();
    let version = env!("CARGO_PKG_VERSION");

    let left = format!("  backend  {}  ~2", branch);
    let right = format!("{}  |  v{}", time_str, version);
    let right_len = right.len() as u16;
    let left_len = left.len() as u16;
    let gap = area.width.saturating_sub(left_len + right_len);

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::Gray)),
        Span::raw(" ".repeat(gap as usize)),
        Span::styled(right, Style::default().fg(Color::DarkGray)),
    ]);
    Paragraph::new(line).render(area, buf);
}

fn get_git_branch() -> String {
    if let Ok(head) = std::fs::read_to_string(".git/HEAD") {
        if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
            return branch.trim().to_string();
        }
    }
    "main".to_string()
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
                    frame.render_widget(SettingsFormWidget { state: &app.settings }, area);
                }
                AppState::Running { .. } => {
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

                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(0), Constraint::Length(1)])
                        .split(area);

                    frame.render_widget(
                        OutputPane {
                            lines: &app.output_lines,
                            scroll_offset: app.output_scroll,
                            title: output_title,
                            active: output_active,
                        },
                        vertical[0],
                    );

                    Paragraph::new(Line::from(Span::styled(
                        "↑↓ scroll  •  Esc back to menu",
                        Style::default().fg(Color::DarkGray),
                    )))
                    .render(vertical[1], frame.buffer_mut());
                }
                _ => {
                    // Layout: [image+text row] [status] [menu] [footer]
                    let menu_items_count = crate::tui::app::MENU_ITEMS.len() as u16;
                    let menu_height = 1 + menu_items_count; // header + items
                    let logo_row_height = LOGO_ROWS as u16;
                    let status_height = 1u16;
                    let footer_height = 1u16;

                    let vertical = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(logo_row_height),
                            Constraint::Length(1), // tagline
                            Constraint::Length(status_height),
                            Constraint::Length(menu_height),
                            Constraint::Min(0),
                            Constraint::Length(footer_height),
                        ])
                        .split(area);

                    // Logo row: image on left (if available), big text on right
                    let img_col_width = if app.logo_ansi.is_empty() { 0 } else { LOGO_COLS as u16 };
                    let logo_horiz = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(img_col_width),
                            Constraint::Min(0),
                        ])
                        .split(vertical[0]);

                    if !app.logo_ansi.is_empty() {
                        frame.render_widget(
                            ChafaImageWidget { lines: &app.logo_ansi },
                            logo_horiz[0],
                        );
                    }

                    // Vertically center big text within the logo row height
                    let text_top_pad = (logo_row_height.saturating_sub(BIG_TEXT_HEIGHT)) / 2;
                    let text_area = Rect {
                        x: logo_horiz[1].x,
                        y: logo_horiz[1].y + text_top_pad,
                        width: logo_horiz[1].width,
                        height: BIG_TEXT_HEIGHT,
                    };
                    frame.render_widget(BigTextWidget, text_area);

                    // Tagline
                    let tagline_text = "The lazy engineer's Swiss Army knife";
                    let tagline_len = tagline_text.len() as u16;
                    let tagline_x = area.x + area.width.saturating_sub(tagline_len) / 2;
                    Paragraph::new(Line::from(Span::styled(
                        tagline_text,
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    )))
                    .render(
                        Rect { x: tagline_x, y: vertical[1].y, width: tagline_len.min(area.width), height: 1 },
                        frame.buffer_mut(),
                    );

                    // Status bar
                    render_status_bar(vertical[2], frame.buffer_mut());

                    // Menu (left-padded 2 cols)
                    let menu_area = Rect {
                        x: vertical[3].x + 2,
                        y: vertical[3].y,
                        width: vertical[3].width.saturating_sub(2),
                        height: vertical[3].height,
                    };
                    frame.render_widget(MenuWidget { selected: app.menu_selection }, menu_area);

                    // Footer
                    Paragraph::new(Line::from(Span::styled(
                        "←↓↑→ navigate • enter submit",
                        Style::default().fg(Color::DarkGray),
                    )))
                    .render(vertical[5], frame.buffer_mut());
                }
            }
        })?;

        loop {
            match rx.try_recv() {
                Ok(event) => handle_app_event(app, event),
                Err(_) => break,
            }
        }

        if let Some(term_event) = poll_crossterm_event() {
            let is_enter = events::is_enter_key(&term_event);
            let should_quit = handle_key(app, term_event);

            if should_quit {
                break;
            }

            if is_enter {
                if let AppState::Menu = &app.state {
                    match app.menu_selection {
                        0 => {
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
                            app.state = AppState::Settings;
                        }
                        3 => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }

    Ok(())
}

async fn run_time_tui(
    args: crate::cli::TimeArgs,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
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

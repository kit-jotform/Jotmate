use anyhow::Result;
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Write};

use crate::cli::{SyncArgs, TimeArgs};

const LOGO: &str = r#"     ██╗ ██████╗ ████████╗███╗   ███╗ █████╗ ████████╗███████╗
     ██║██╔═══██╗╚══██╔══╝████╗ ████║██╔══██╗╚══██╔══╝██╔════╝
     ██║██║   ██║   ██║   ██╔████╔██║███████║   ██║   █████╗
██   ██║██║   ██║   ██║   ██║╚██╔╝██║██╔══██║   ██║   ██╔══╝
╚█████╔╝╚██████╔╝   ██║   ██║ ╚═╝ ██║██║  ██║   ██║   ███████╗
 ╚════╝  ╚═════╝    ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚══════╝"#;

const LOGO_SMALL: &str = r#"╦╔═╗╔╦╗╔╦╗╔═╗╔╦╗╔═╗
║║ ║ ║ ║║║╠═╣ ║ ║╣
╚╝╚═╝ ╩ ╩ ╩╩ ╩ ╩ ╚═╝"#;
const ICON_ANSI: &str = include_str!("../../assets/icon.txt");

const C_PRIMARY: &str = "\x1b[38;5;141m";
const C_SECONDARY: &str = "\x1b[38;5;87m";
const C_SUCCESS: &str = "\x1b[38;5;82m";
const C_MUTED: &str = "\x1b[38;5;245m";
const C_ACCENT: &str = "\x1b[38;5;219m";
const C_TEXT: &str = "\x1b[38;5;255m";
const C_RESET: &str = "\x1b[0m";

#[derive(Clone, Copy)]
enum MainChoice {
    Sync,
    Time,
    Settings,
    Exit,
}

struct TerminalSession;

impl TerminalSession {
    fn new() -> Result<Self> {
        enter_alt_screen();
        hide_cursor();
        enable_raw_mode()?;
        Ok(Self)
    }

    fn pause_for_editor(&self) -> Result<()> {
        disable_raw_mode()?;
        show_cursor();
        leave_alt_screen();
        Ok(())
    }

    fn resume_after_editor(&self) -> Result<()> {
        enter_alt_screen();
        hide_cursor();
        enable_raw_mode()?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        show_cursor();
        leave_alt_screen();
    }
}

pub async fn run_interactive() -> Result<()> {
    let session = TerminalSession::new()?;

    let result = async {
        loop {
            match show_main_menu()? {
                MainChoice::Sync => {
                    show_tool_header("Sync", "Sync repos to upstream")?;
                    crate::sync::run(SyncArgs::default()).await?;
                    if !show_done_screen("Sync")? {
                        break;
                    }
                }
                MainChoice::Time => {
                    show_tool_header("Time Doctor", "Track your work hours")?;
                    crate::time::run(TimeArgs::default()).await?;
                    if !show_done_screen("Time Doctor")? {
                        break;
                    }
                }
                MainChoice::Settings => {
                    session.pause_for_editor()?;
                    run_settings().await?;
                    session.resume_after_editor()?;
                }
                MainChoice::Exit => break,
            }
        }

        Ok(())
    }
    .await;

    drop(session);
    println!("\n  See you later, engineer. Ship it!\n");

    result
}

pub async fn run_settings() -> Result<()> {
    let path = crate::config::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        let default = crate::config::Config::default();
        crate::config::save(&default)?;
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    std::process::Command::new(&editor).arg(&path).status()?;
    Ok(())
}

fn show_main_menu() -> Result<MainChoice> {
    let items = [
        (MainChoice::Sync, "Sync", "Sync repos to upstream"),
        (MainChoice::Time, "Time Doctor", "Track your work hours"),
        (MainChoice::Settings, "Settings", "Configure jotmate"),
        (MainChoice::Exit, "Exit", "Leave jotmate"),
    ];

    let mut selected = 0usize;

    loop {
        clear_screen();
        newline();
        print_main_logo_with_icon();
        print_centered_colored("The lazy engineer's Swiss Army knife", C_MUTED, false);
        newline();
        print_centered_colored(&build_context_line(), C_MUTED, false);
        newline();
        print_centered_colored("─────────────────────────────────────────────────", C_MUTED, false);
        newline();
        print_centered_colored("SELECT TOOL  (↑↓ navigate · Enter select · Esc exit)", C_SECONDARY, true);
        newline();

        print_main_menu_items(&items, selected);

        io::stdout().flush()?;

        match read_key()? {
            KeyCode::Up => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Down => {
                if selected + 1 < items.len() {
                    selected += 1;
                }
            }
            KeyCode::Enter => return Ok(items[selected].0),
            KeyCode::Esc => return Ok(MainChoice::Exit),
            _ => {}
        }
    }
}

fn show_tool_header(tool_name: &str, tool_desc: &str) -> Result<()> {
    clear_screen();
    newline();
    print_centered_colored(LOGO_SMALL, C_PRIMARY, true);
    newline();
    print_centered_colored(tool_name, C_ACCENT, true);
    print_centered_colored(tool_desc, C_MUTED, false);
    newline();
    print_centered_colored("─────────────────────────────────────────────────", C_MUTED, false);
    newline();
    io::stdout().flush()?;
    Ok(())
}

fn show_done_screen(tool_name: &str) -> Result<bool> {
    show_tool_header(tool_name, "Completed")?;
    print_centered_colored("DONE!", C_SUCCESS, true);
    newline();
    print_centered_colored("Enter: main menu   ·   Esc: exit jotmate", C_MUTED, false);
    io::stdout().flush()?;

    loop {
        match read_key()? {
            KeyCode::Enter => return Ok(true),
            KeyCode::Esc => return Ok(false),
            _ => {}
        }
    }
}

fn read_key() -> Result<KeyCode> {
    loop {
        if let Event::Key(key_event) = event::read()? {
            return Ok(key_event.code);
        }
    }
}

fn build_context_line() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!("{}  |  v{}", Local::now().format("%H:%M"), version)
}

fn print_main_logo_with_icon() {
    let icon_lines = icon_lines();
    let logo_lines: Vec<&str> = LOGO.lines().collect();

    let icon_col_width = 16usize;
    let logo_offset = 1usize;
    let max_lines = icon_lines.len().max(logo_lines.len() + logo_offset);

    let mut block_lines = Vec::with_capacity(max_lines);
    for i in 0..max_lines {
        let icon_part = icon_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let logo_idx = i as isize - logo_offset as isize;
        let logo_part = if logo_idx >= 0 {
            logo_lines.get(logo_idx as usize).copied().unwrap_or("")
        } else {
            ""
        };

        let icon_width = display_width(icon_part);
        let pad = " ".repeat(icon_col_width.saturating_sub(icon_width));
        block_lines.push(format!("{icon_part}{pad}{C_TEXT}\x1b[1m{logo_part}{C_RESET}"));
    }

    print_centered_block(&block_lines, false);
}

fn print_main_menu_items(items: &[(MainChoice, &str, &str)], selected: usize) {
    let title_w = items.iter().map(|(_, title, _)| title.chars().count()).max().unwrap_or(0);
    let lines: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(idx, (_, title, desc))| {
            let marker = if idx == selected { "▸" } else { " " };
            format!("  {marker} {:<title_w$}  ─ {}", title, desc, title_w = title_w)
        })
        .collect();

    print_centered_block(
        &lines,
        true,
    );
}

fn print_centered_block(lines: &[String], selected_first_colorized: bool) {
    let block_width = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
    let left = term_width().saturating_sub(block_width) / 2;
    let pad = " ".repeat(left);

    for (idx, line) in lines.iter().enumerate() {
        if selected_first_colorized && line.contains("▸") {
            print!("{pad}{C_ACCENT}\x1b[1m{line}{C_RESET}\r\n");
        } else if selected_first_colorized {
            print!("{pad}{C_MUTED}{line}{C_RESET}\r\n");
        } else {
            let _ = idx; // keep loop structure consistent for both branches
            print!("{pad}{line}\r\n");
        }
    }
}

fn icon_lines() -> Vec<String> {
    ICON_ANSI
        .lines()
        .filter_map(|line| {
            let clean = line.replace("\x1b[?25l", "").replace("\x1b[?25h", "");
            if clean.trim().is_empty() {
                None
            } else {
                Some(clean)
            }
        })
        .collect()
}

fn print_centered_colored(text: &str, color: &str, bold: bool) {
    let width = term_width();
    for line in text.lines() {
        let len = display_width(line);
        let pad = width.saturating_sub(len) / 2;
        let prefix = " ".repeat(pad);
        if bold {
            print!("{}{}\x1b[1m{}{}\r\n", prefix, color, line, C_RESET);
        } else {
            print!("{}{}{}{}\r\n", prefix, color, line, C_RESET);
        }
    }
}

fn display_width(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            while let Some(next) = chars.next() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn term_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .ok()
        .filter(|w| *w > 0)
        .unwrap_or(80)
}

fn newline() {
    print!("\r\n");
}

fn enter_alt_screen() {
    print!("\x1b[?1049h");
    let _ = io::stdout().flush();
}

fn leave_alt_screen() {
    print!("\x1b[?1049l");
    let _ = io::stdout().flush();
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

fn hide_cursor() {
    print!("\x1b[?25l");
    let _ = io::stdout().flush();
}

fn show_cursor() {
    print!("\x1b[?25h");
    let _ = io::stdout().flush();
}

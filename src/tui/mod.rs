use anyhow::Result;
use chrono::Local;
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

#[derive(Clone, Copy)]
enum MainChoice {
    Sync,
    Time,
    Settings,
    Exit,
}

pub async fn run_interactive() -> Result<()> {
    hide_cursor();

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
                    show_cursor();
                    run_settings().await?;
                    hide_cursor();
                }
                MainChoice::Exit => break,
            }
        }

        Ok(())
    }
    .await;

    show_cursor();
    leave_alt_screen();
    println!("\n  See you later, engineer. Ship it!\n");

    result
}

pub async fn run_settings() -> Result<()> {
    // Open config file in $EDITOR, creating it first if needed
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
    enter_alt_screen();
    clear_screen();

    let context = build_context_line();

    println!();
    print_centered(LOGO);
    println!("\nThe lazy engineer's Swiss Army knife\n");
    println!("{}", context);
    println!("\n---------------------------------------------\n");
    println!("1. Sync         - Sync repos to upstream");
    println!("2. Time Doctor  - Track your work hours");
    println!("3. Settings     - Configure jotmate");
    println!("4. Exit");
    print!("\nSelect tool [1-4]: ");
    io::stdout().flush()?;

    let input = read_line_trimmed()?;
    let choice = match input.as_str() {
        "1" => MainChoice::Sync,
        "2" => MainChoice::Time,
        "3" => MainChoice::Settings,
        _ => MainChoice::Exit,
    };

    Ok(choice)
}

fn show_tool_header(tool_name: &str, tool_desc: &str) -> Result<()> {
    enter_alt_screen();
    clear_screen();

    println!();
    print_centered(LOGO_SMALL);
    println!();
    print_centered(tool_name);
    println!();
    print_centered(tool_desc);
    println!("\n---------------------------------------------\n");

    io::stdout().flush()?;
    Ok(())
}

fn show_done_screen(tool_name: &str) -> Result<bool> {
    show_tool_header(tool_name, "Completed")?;
    print_centered("DONE!");
    println!();
    println!("Press Enter for main menu, or type q to quit: ");
    io::stdout().flush()?;

    let input = read_line_trimmed()?;
    Ok(!matches!(input.as_str(), "q" | "Q" | "quit" | "exit"))
}

fn build_context_line() -> String {
    let version = env!("CARGO_PKG_VERSION");

    format!("{}  |  v{}", Local::now().format("%H:%M"), version)
}

fn read_line_trimmed() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn print_centered(text: &str) {
    let width = term_width();
    for line in text.lines() {
        let len = line.chars().count();
        let pad = width.saturating_sub(len) / 2;
        let prefix = " ".repeat(pad);
        println!("{}{}", prefix, line);
    }
}

fn term_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(80)
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

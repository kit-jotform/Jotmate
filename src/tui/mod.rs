use anyhow::Result;
use std::io::Write;

const TUI_SH: &str = include_str!("../../scripts/tui.sh");

fn exec_tui_sh() -> Result<()> {
    let mut tmpfile = tempfile::NamedTempFile::new()?;
    tmpfile.write_all(TUI_SH.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(tmpfile.path())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(tmpfile.path(), perms)?;
    }

    let current_exe = std::env::current_exe()?;
    let status = std::process::Command::new("bash")
        .arg(tmpfile.path())
        .arg(&current_exe)
        .status()?;

    drop(tmpfile); // keep alive until here

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

pub async fn run_interactive() -> Result<()> {
    exec_tui_sh()
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
    std::process::Command::new(&editor)
        .arg(&path)
        .status()?;
    Ok(())
}

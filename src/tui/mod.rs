use anyhow::{bail, Result};
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

    drop(tmpfile);

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

pub async fn run_interactive() -> Result<()> {
    exec_tui_sh()
}

/// Called by `jotmate settings` — the shell TUI handles UI; this is a no-op placeholder
/// (the real settings UI lives in tui.sh as run_settings()).
pub async fn run_settings() -> Result<()> {
    Ok(())
}

// ── Data operations for the shell settings TUI ───────────────────────────────

/// Print all settings as key=value lines for the shell to parse.
/// Repos are printed as: repo.<name>.url=... and repo.<name>.enabled=...
pub fn settings_get() -> Result<()> {
    let config = crate::config::load()?;
    println!("sync_all_by_default={}", config.sync.sync_all_by_default);
    println!("use_cache={}", config.sync.use_cache);
    for repo in &config.sync.upstream_repos {
        println!("repo.{}.url={}", repo.name, repo.url);
        println!("repo.{}.enabled={}", repo.name, repo.enabled);
    }
    Ok(())
}

/// Toggle a boolean field. Supported: sync_all_by_default, use_cache
pub fn settings_toggle(field: &str) -> Result<()> {
    let mut config = crate::config::load()?;
    match field {
        "sync_all_by_default" => {
            config.sync.sync_all_by_default = !config.sync.sync_all_by_default;
            println!("{}", config.sync.sync_all_by_default);
        }
        "use_cache" => {
            config.sync.use_cache = !config.sync.use_cache;
            println!("{}", config.sync.use_cache);
        }
        other => bail!("Unknown field: {}", other),
    }
    crate::config::save(&config)?;
    Ok(())
}

/// Add a new upstream repo.
pub fn settings_add_repo(url: &str, name: &str) -> Result<()> {
    let mut config = crate::config::load()?;
    if config.sync.upstream_repos.iter().any(|r| r.name == name) {
        bail!("A repo named '{}' already exists", name);
    }
    config.sync.upstream_repos.push(crate::config::UpstreamRepo::new(url, name));
    crate::config::save(&config)?;
    Ok(())
}

/// Remove an upstream repo by name.
pub fn settings_remove_repo(name: &str) -> Result<()> {
    let mut config = crate::config::load()?;
    let before = config.sync.upstream_repos.len();
    config.sync.upstream_repos.retain(|r| r.name != name);
    if config.sync.upstream_repos.len() == before {
        bail!("No repo named '{}' found", name);
    }
    crate::config::save(&config)?;
    Ok(())
}

/// Toggle the enabled flag on an upstream repo by name.
pub fn settings_toggle_repo(name: &str) -> Result<()> {
    let mut config = crate::config::load()?;
    match config.sync.upstream_repos.iter_mut().find(|r| r.name == name) {
        Some(repo) => {
            repo.enabled = !repo.enabled;
            println!("{}", repo.enabled);
            crate::config::save(&config)?;
        }
        None => bail!("No repo named '{}' found", name),
    }
    Ok(())
}

pub mod cache;
pub mod discover;
pub mod runner;

use anyhow::Result;

use crate::cli::SyncArgs;
use cache::compute_github_base;

pub async fn run(args: SyncArgs) -> Result<()> {
    let paths = resolve_repo_paths()?;
    let github_base = compute_github_base(&paths).ok_or_else(|| {
        anyhow::anyhow!(
            "Repositories do not share a common parent directory. \
             Please ensure all repos are cloned under the same directory."
        )
    })?;

    runner::run_cli(&args, &github_base)
}

pub async fn run_tui(
    args: SyncArgs,
    tx: tokio::sync::mpsc::Sender<crate::tui::events::AppEvent>,
) -> Result<()> {
    let paths = resolve_repo_paths()?;
    let github_base = compute_github_base(&paths).ok_or_else(|| {
        anyhow::anyhow!("Repositories do not share a common parent directory.")
    })?;

    runner::run_tui(&args, &github_base, tx).await
}

fn resolve_repo_paths() -> Result<std::collections::HashMap<String, std::path::PathBuf>> {
    // Try cache first
    if let Some(cached) = cache::load() {
        if cache::is_valid(&cached) {
            return Ok(cached.paths);
        }
        eprintln!("Cached repo paths are invalid, rediscovering...");
        cache::invalidate();
    }

    // Discover via fd
    let discovered = discover::discover_and_cache()?;
    Ok(discovered.paths)
}

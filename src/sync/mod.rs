pub mod cache;
pub mod discover;
pub mod runner;

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli::SyncArgs;
use crate::config::UpstreamRepo;
use cache::compute_github_base;

pub async fn run(args: SyncArgs) -> Result<()> {
    let config = crate::config::load()?;
    let upstream_repos = &config.sync.upstream_repos;
    let paths = resolve_repo_paths(upstream_repos, config.sync.use_cache)?;
    let github_base = compute_github_base(&paths).ok_or_else(|| {
        anyhow::anyhow!(
            "Repositories do not share a common parent directory. \
             Please ensure all repos are cloned under the same directory."
        )
    })?;

    runner::run_cli(&args, &github_base)
}

fn resolve_repo_paths(
    upstream_repos: &[UpstreamRepo],
    use_cache: bool,
) -> Result<HashMap<String, PathBuf>> {
    let enabled_names: Vec<&str> = upstream_repos
        .iter()
        .filter(|r| r.enabled)
        .map(|r| r.name.as_str())
        .collect();

    if use_cache {
        if let Some(cached) = cache::load() {
            if cache::is_valid(&cached, &enabled_names) {
                return Ok(cached.paths);
            }
            eprintln!("Cached repo paths are invalid, rediscovering...");
            cache::invalidate();
        }
    }

    // Discover via fd
    let discovered = discover::discover_and_cache(upstream_repos)?;
    Ok(discovered.paths)
}

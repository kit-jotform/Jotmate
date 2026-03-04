use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::cache::{self, RepoPathsCache, KNOWN_PROJECTS};
use crate::error::AppError;

const KNOWN_UPSTREAMS: &[(&str, &str)] = &[
    ("https://github.com/jotform/frontend.git", "frontend"),
    ("https://github.com/jotform/vendors.git", "vendors"),
    ("https://github.com/jotform/backend.git", "backend"),
    ("https://github.com/jotform/Jotform3.git", "Jotform3"),
    ("https://github.com/jotform/core.git", "core"),
    // Also match SSH remotes
    ("git@github.com:jotform/frontend.git", "frontend"),
    ("git@github.com:jotform/vendors.git", "vendors"),
    ("git@github.com:jotform/backend.git", "backend"),
    ("git@github.com:jotform/Jotform3.git", "Jotform3"),
    ("git@github.com:jotform/core.git", "core"),
];

pub fn discover_all_git_repos() -> Result<Vec<PathBuf>> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;

    // Check fd is available
    let check = Command::new("fd").arg("--version").output();
    if check.is_err() || !check.unwrap().status.success() {
        return Err(AppError::FdNotFound.into());
    }

    let output = Command::new("fd")
        .args(["-H", "-t", "d", "^.git$", home.to_str().unwrap()])
        .output()
        .context("Failed to run fd")?;

    if !output.status.success() {
        anyhow::bail!("fd exited with error: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let repos: Vec<PathBuf> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let git_dir = PathBuf::from(line.trim());
            git_dir.parent().map(|p| p.to_path_buf())
        })
        .collect();

    Ok(repos)
}

pub fn get_upstream_url(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", repo_path.to_str()?, "remote", "get-url", "upstream"])
        .output()
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !url.is_empty() {
            return Some(url);
        }
    }
    None
}

pub fn match_repos_to_projects(repo_roots: &[PathBuf]) -> Result<HashMap<String, PathBuf>> {
    let mut found: HashMap<String, PathBuf> = HashMap::new();

    for repo_path in repo_roots {
        if let Some(url) = get_upstream_url(repo_path) {
            let url_trimmed = url.trim_end_matches('/');
            for (upstream_url, project_name) in KNOWN_UPSTREAMS {
                if url_trimmed == *upstream_url || url_trimmed == upstream_url.trim_end_matches(".git") {
                    found
                        .entry(project_name.to_string())
                        .or_insert_with(|| repo_path.clone());
                    break;
                }
            }
        }
        // Stop early if all found
        if found.len() == KNOWN_PROJECTS.len() {
            break;
        }
    }

    let missing: Vec<&str> = KNOWN_PROJECTS
        .iter()
        .filter(|p| !found.contains_key(**p))
        .copied()
        .collect();

    if !missing.is_empty() {
        anyhow::bail!(
            "Could not find local paths for: {}. \
             Ensure you have cloned these repos with an 'upstream' remote configured.",
            missing.join(", ")
        );
    }

    Ok(found)
}

pub fn discover_and_cache() -> Result<RepoPathsCache> {
    println!("Discovering git repositories (this may take a moment)...");
    let repos = discover_all_git_repos()?;
    println!("Found {} git repos, matching against known upstreams...", repos.len());

    let paths = match_repos_to_projects(&repos)?;

    println!("All repositories located:");
    for (project, path) in &paths {
        println!("  {project}: {}", path.display());
    }

    let cache = RepoPathsCache {
        version: 1,
        cached_at: Utc::now(),
        paths,
    };

    cache::save(&cache)?;
    Ok(cache)
}

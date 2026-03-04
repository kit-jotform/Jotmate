use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CACHE_VERSION: u32 = 1;

pub const KNOWN_PROJECTS: &[&str] = &["Jotform3", "vendors", "core", "backend", "frontend"];

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoPathsCache {
    pub version: u32,
    pub cached_at: DateTime<Utc>,
    pub paths: HashMap<String, PathBuf>,
}

pub fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("jotmate")
        .join("repo_paths.json")
}

pub fn load() -> Option<RepoPathsCache> {
    let path = cache_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let cache: RepoPathsCache = serde_json::from_str(&content).ok()?;
    if cache.version != CACHE_VERSION {
        return None;
    }
    Some(cache)
}

pub fn save(cache: &RepoPathsCache) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create cache directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(cache)?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write cache to {}", path.display()))?;
    Ok(())
}

pub fn is_valid(cache: &RepoPathsCache) -> bool {
    for project in KNOWN_PROJECTS {
        match cache.paths.get(*project) {
            Some(p) if p.exists() => {}
            _ => return false,
        }
    }
    true
}

pub fn invalidate() {
    let _ = std::fs::remove_file(cache_path());
}

pub fn compute_github_base(paths: &HashMap<String, PathBuf>) -> Option<PathBuf> {
    // Find common parent: all paths must be <parent>/<project_name>
    let mut candidate: Option<PathBuf> = None;
    for (project, path) in paths {
        let parent = path.parent()?;
        let filename = path.file_name()?;
        if filename != Path::new(project).as_os_str() {
            return None;
        }
        match &candidate {
            None => candidate = Some(parent.to_path_buf()),
            Some(c) if c != parent => return None,
            _ => {}
        }
    }
    candidate
}

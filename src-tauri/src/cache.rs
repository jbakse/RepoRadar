use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use crate::models::RepoSummary;

/// Fingerprint of a repo's git state — derived from mtimes of key git files.
/// Any change to these files means the repo state may have changed.
#[derive(Debug, Clone, PartialEq)]
pub struct RepoFingerprint {
    pub git_index: Option<SystemTime>,
    pub git_head: Option<SystemTime>,
    pub refs_heads: Option<SystemTime>,
    pub refs_remotes: Option<SystemTime>,
    pub packed_refs: Option<SystemTime>,
    pub repo_root: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub summary: RepoSummary,
    pub fingerprint: RepoFingerprint,
}

pub struct RepoCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
}

impl RepoCache {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Compute a fingerprint for a repo by stat-ing key git files.
    /// Handles both normal .git directories and .git files (worktrees/submodules).
    pub fn compute_fingerprint(repo_path: &Path) -> RepoFingerprint {
        let git_dir = resolve_git_dir(repo_path);

        RepoFingerprint {
            git_index: mtime(&git_dir.join("index")),
            git_head: mtime(&git_dir.join("HEAD")),
            refs_heads: max_mtime_under(&git_dir.join("refs").join("heads")),
            refs_remotes: max_mtime_under(&git_dir.join("refs").join("remotes")),
            packed_refs: mtime(&git_dir.join("packed-refs")),
            repo_root: mtime(repo_path),
        }
    }

    /// Returns true if the repo should be rescanned (no cache entry, failed stat, or mtime differs).
    pub fn is_stale(&self, path: &str, fresh: &RepoFingerprint) -> bool {
        let entries = self.entries.lock().unwrap();
        match entries.get(path) {
            None => true,
            Some(entry) => {
                let cached = &entry.fingerprint;
                // Any None in the fresh fingerprint means we couldn't stat — treat as stale
                if fresh.git_index.is_none()
                    || fresh.git_head.is_none()
                    || fresh.refs_heads.is_none()
                    || fresh.refs_remotes.is_none()
                {
                    return true;
                }
                *cached != *fresh
            }
        }
    }

    /// Insert or replace a cache entry.
    pub fn update(&self, path: &str, summary: RepoSummary, fingerprint: RepoFingerprint) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(
            path.to_string(),
            CacheEntry {
                summary,
                fingerprint,
            },
        );
    }

    /// Retrieve a cached summary.
    pub fn get(&self, path: &str) -> Option<RepoSummary> {
        let entries = self.entries.lock().unwrap();
        entries.get(path).map(|e| e.summary.clone())
    }

    /// Remove a single cache entry by path.
    pub fn evict(&self, path: &str) {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(path);
    }

    /// Remove entries for repos no longer in the known set.
    pub fn evict_not_in(&self, known_paths: &[String]) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|k, _| known_paths.contains(k));
    }

    /// Return all cached repo paths.
    pub fn all_paths(&self) -> Vec<String> {
        let entries = self.entries.lock().unwrap();
        entries.keys().cloned().collect()
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }
}

/// Resolve the actual .git directory. For worktrees/submodules, .git is a file
/// containing "gitdir: <path>". We follow that to the real git directory.
fn resolve_git_dir(repo_path: &Path) -> PathBuf {
    let dot_git = repo_path.join(".git");
    if dot_git.is_file() {
        // .git file — read the gitdir path
        if let Ok(content) = fs::read_to_string(&dot_git) {
            let trimmed = content.trim();
            if let Some(gitdir) = trimmed.strip_prefix("gitdir: ") {
                let gitdir_path = PathBuf::from(gitdir);
                if gitdir_path.is_absolute() {
                    return gitdir_path;
                } else {
                    // Relative to repo root
                    return repo_path.join(gitdir_path);
                }
            }
        }
    }
    dot_git
}

/// Get the mtime of a path, returning None if stat fails.
fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

/// Return the later of two optional timestamps.
fn latest(a: Option<SystemTime>, b: Option<SystemTime>) -> Option<SystemTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Recursively find the most recent mtime under a directory.
/// These dirs (refs/heads, refs/remotes) contain only a handful of files,
/// so the recursion is cheap.
fn max_mtime_under(dir: &Path) -> Option<SystemTime> {
    let mut max = mtime(dir);
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return max,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        max = latest(max, mtime(&path));
        if path.is_dir() {
            max = latest(max, max_mtime_under(&path));
        }
    }
    max
}

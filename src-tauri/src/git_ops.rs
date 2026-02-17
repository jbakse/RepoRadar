use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::models::*;

/// Resolve the git binary path once and cache it.
/// Checks GIT_PATH env var, then `which git`, then common locations.
pub fn git_binary() -> &'static PathBuf {
    static GIT_BIN: OnceLock<PathBuf> = OnceLock::new();
    GIT_BIN.get_or_init(|| {
        // 1. Check GIT_PATH env var
        if let Ok(path) = std::env::var("GIT_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return p;
            }
        }

        // 2. Try `which git`
        if let Ok(output) = Command::new("which").arg("git").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    let p = PathBuf::from(&path);
                    if p.exists() {
                        return p;
                    }
                }
            }
        }

        // 3. Common fallback locations
        for candidate in &["/usr/bin/git", "/usr/local/bin/git", "/opt/homebrew/bin/git"] {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return p;
            }
        }

        // Last resort: just use "git" and hope PATH works
        PathBuf::from("git")
    })
}

fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(git_binary())
        .args(["-C", &repo_path.to_string_lossy()])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(stderr)
    }
}

pub fn get_last_commit(repo_path: &Path) -> Option<CommitInfo> {
    let output = run_git(
        repo_path,
        &["log", "-1", "--format=%H%n%s%n%an%n%aI"],
    )
    .ok()?;

    let lines: Vec<&str> = output.lines().collect();
    if lines.len() >= 4 {
        Some(CommitInfo {
            hash: lines[0][..7.min(lines[0].len())].to_string(),
            subject: lines[1].to_string(),
            author: lines[2].to_string(),
            timestamp: lines[3].to_string(),
        })
    } else {
        None
    }
}

pub fn get_created_date(repo_path: &Path) -> Option<String> {
    // Try earliest commit date first
    if let Ok(output) = run_git(
        repo_path,
        &["log", "--reverse", "--format=%aI", "--max-count=1"],
    ) {
        if !output.is_empty() {
            return Some(output);
        }
    }

    // Fall back to filesystem birth time
    if let Ok(metadata) = std::fs::metadata(repo_path) {
        if let Ok(created) = metadata.created() {
            let datetime: chrono::DateTime<chrono::Local> = created.into();
            return Some(datetime.to_rfc3339());
        }
    }

    None
}

pub fn get_uncommitted_and_sync(repo_path: &Path) -> (UncommittedChanges, SyncStatus) {
    let mut changes = UncommittedChanges::default();
    let mut sync = SyncStatus::default();

    if let Ok(output) = run_git(repo_path, &["status", "--porcelain=v2", "-b"]) {
        for line in output.lines() {
            if line.starts_with("# branch.head") {
                let branch = line.split_whitespace().last().unwrap_or("").to_string();
                if branch == "(detached)" {
                    sync.detached = true;
                } else {
                    sync.branch = Some(branch);
                }
            } else if line.starts_with("# branch.upstream") {
                sync.upstream = line.split_whitespace().last().map(|s| s.to_string());
            } else if line.starts_with("# branch.ab") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    sync.ahead = parts[2]
                        .trim_start_matches('+')
                        .parse()
                        .unwrap_or(0);
                    sync.behind = parts[3]
                        .trim_start_matches('-')
                        .parse()
                        .unwrap_or(0);
                }
            } else if line.starts_with("1 ") || line.starts_with("2 ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let xy = parts[1];
                    if xy.len() >= 2 {
                        let x = xy.chars().nth(0).unwrap_or('.');
                        let y = xy.chars().nth(1).unwrap_or('.');
                        if x != '.' {
                            changes.staged += 1;
                        }
                        if y != '.' {
                            changes.unstaged += 1;
                        }
                    }
                }
            } else if line.starts_with("? ") {
                changes.untracked += 1;
            } else if line.starts_with("u ") {
                changes.unstaged += 1;
            }
        }
    }

    // Get branches without upstream
    if let Ok(output) = run_git(repo_path, &["for-each-ref", "--format=%(refname:short) %(upstream)", "refs/heads/"]) {
        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 1 || (parts.len() == 2 && parts[1].is_empty()) {
                sync.branches_without_upstream.push(parts[0].to_string());
            }
        }
    }

    (changes, sync)
}

pub fn get_remotes(repo_path: &Path) -> Vec<RemoteInfo> {
    let mut remotes = Vec::new();

    if let Ok(output) = run_git(repo_path, &["remote", "-v"]) {
        let mut seen = std::collections::HashMap::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();
                let kind = parts[2].trim_matches(|c| c == '(' || c == ')');

                let entry = seen.entry(name.clone()).or_insert_with(|| RemoteInfo {
                    name: name.clone(),
                    fetch_url: String::new(),
                    push_url: String::new(),
                });

                match kind {
                    "fetch" => entry.fetch_url = url,
                    "push" => entry.push_url = url,
                    _ => {}
                }
            }
        }

        remotes = seen.into_values().collect();
        remotes.sort_by(|a, b| a.name.cmp(&b.name));
    }

    remotes
}

pub fn get_branches(repo_path: &Path) -> Vec<BranchInfo> {
    let mut branches = Vec::new();

    // Get current branch
    let current = run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_default();

    if let Ok(output) = run_git(
        repo_path,
        &[
            "for-each-ref",
            "--format=%(refname:short)\t%(upstream:short)\t%(upstream:track)",
            "refs/heads/",
        ],
    ) {
        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.is_empty() {
                continue;
            }

            let name = parts[0].to_string();
            let upstream = if parts.len() > 1 && !parts[1].is_empty() {
                Some(parts[1].to_string())
            } else {
                None
            };

            let (ahead, behind) = if parts.len() > 2 {
                parse_track_info(parts[2])
            } else {
                (0, 0)
            };

            branches.push(BranchInfo {
                is_current: name == current,
                name,
                upstream,
                ahead,
                behind,
            });
        }
    }

    branches
}

fn parse_track_info(track: &str) -> (u32, u32) {
    let mut ahead = 0u32;
    let mut behind = 0u32;

    if track.contains("ahead") {
        if let Some(n) = track
            .split("ahead ")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
        {
            ahead = n;
        }
    }

    if track.contains("behind") {
        if let Some(n) = track
            .split("behind ")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
        {
            behind = n;
        }
    }

    (ahead, behind)
}

pub fn get_last_file_edited(repo_path: &Path, include_untracked: bool) -> Option<FileEditInfo> {
    let files_output = if include_untracked {
        // List all files (tracked + untracked, excluding ignored)
        run_git(repo_path, &["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
    } else {
        run_git(repo_path, &["ls-files", "-z"])
    };

    let output = files_output.ok()?;
    if output.is_empty() {
        return None;
    }

    let mut latest: Option<(String, std::time::SystemTime)> = None;

    for file in output.split('\0') {
        if file.is_empty() {
            continue;
        }

        let full_path = repo_path.join(file);
        if let Ok(metadata) = std::fs::metadata(&full_path) {
            if let Ok(mtime) = metadata.modified() {
                if latest.is_none() || mtime > latest.as_ref().unwrap().1 {
                    latest = Some((file.to_string(), mtime));
                }
            }
        }
    }

    latest.map(|(path, mtime)| {
        let datetime: chrono::DateTime<chrono::Local> = mtime.into();
        FileEditInfo {
            path,
            mtime: datetime.to_rfc3339(),
        }
    })
}

pub fn get_file_lists(repo_path: &Path) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    if let Ok(output) = run_git(repo_path, &["status", "--porcelain=v2"]) {
        for line in output.lines() {
            if line.starts_with("1 ") || line.starts_with("2 ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let xy = parts[1];
                    let file_path = parts[8..].join(" ");
                    if xy.len() >= 2 {
                        let x = xy.chars().nth(0).unwrap_or('.');
                        let y = xy.chars().nth(1).unwrap_or('.');
                        if x != '.' {
                            staged.push(file_path.clone());
                        }
                        if y != '.' {
                            unstaged.push(file_path);
                        }
                    }
                }
            } else if line.starts_with("? ") {
                let file_path = line[2..].to_string();
                untracked.push(file_path);
            }
        }
    }

    (staged, unstaged, untracked)
}

pub fn get_warnings(repo_path: &Path, sync: &SyncStatus, remotes: &[RemoteInfo]) -> Vec<String> {
    let mut warnings = Vec::new();

    if sync.detached {
        warnings.push("HEAD is detached".to_string());
    }

    if remotes.is_empty() {
        warnings.push("No remote configured".to_string());
    }

    if sync.branch.is_some() && sync.upstream.is_none() && !sync.detached {
        warnings.push(format!(
            "Branch '{}' has no upstream",
            sync.branch.as_deref().unwrap_or("unknown")
        ));
    }

    if sync.ahead > 0 {
        warnings.push(format!("{} commit(s) ahead of upstream", sync.ahead));
    }

    if sync.behind > 0 {
        warnings.push(format!("{} commit(s) behind upstream", sync.behind));
    }

    // Check if there are no commits
    if run_git(repo_path, &["rev-parse", "HEAD"]).is_err() {
        warnings.push("Repository has no commits yet".to_string());
    }

    warnings
}

/// Build a complete repo summary
pub fn build_repo_summary(repo_path: &Path, include_untracked_mtime: bool) -> RepoSummary {
    let folder_name = repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let repo_name = get_repo_name(repo_path).unwrap_or_else(|| folder_name.clone());
    let created = get_created_date(repo_path);
    let last_commit = get_last_commit(repo_path);
    let last_file_edited = get_last_file_edited(repo_path, include_untracked_mtime);
    let (uncommitted, sync_status) = get_uncommitted_and_sync(repo_path);
    let remotes = get_remotes(repo_path);

    let has_high_risk_ignored = false; // Will be checked separately

    RepoSummary {
        path: repo_path.to_string_lossy().to_string(),
        folder_name,
        repo_name,
        created,
        last_commit,
        last_file_edited,
        uncommitted,
        sync_status,
        remotes,
        has_high_risk_ignored,
        error: None,
    }
}

fn get_repo_name(repo_path: &Path) -> Option<String> {
    // Try to derive name from remote origin URL
    if let Ok(url) = run_git(repo_path, &["remote", "get-url", "origin"]) {
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or(&url)
            .trim_end_matches(".git")
            .to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

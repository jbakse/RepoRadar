use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, State};

use crate::cache::RepoCache;
use crate::git_ops;
use crate::models::*;
use crate::persistence;
use crate::risk;
use crate::scanner;

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub config_dir: PathBuf,
    pub current_scan_id: Arc<AtomicU64>,
    pub cache: Arc<RepoCache>,
    pub watcher: Arc<Mutex<Option<crate::watcher::GitWatcher>>>,
    /// Kept alive for its background thread; dropped on app exit.
    #[allow(dead_code)]
    pub poller: Arc<Mutex<Option<crate::poller::WorkTreePoller>>>,
}

impl AppState {
    fn persist(&self) {
        let settings = self.settings.lock().unwrap();
        persistence::save_settings(&self.config_dir, &settings);
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) {
    let old_settings = state.settings.lock().unwrap().clone();
    let untracked_changed =
        old_settings.include_untracked_mtime != settings.include_untracked_mtime;

    let mut s = state.settings.lock().unwrap();
    *s = settings;
    drop(s);
    state.persist();

    // If include_untracked_mtime changed, invalidate all cached data
    if untracked_changed {
        state.cache.clear();
    }
}

#[tauri::command]
pub fn add_folder(state: State<'_, AppState>, path: String) {
    let mut settings = state.settings.lock().unwrap();
    if !settings.folders.contains(&path) {
        settings.folders.push(path);
    }
    let folders = settings.folders.clone();
    drop(settings);
    state.persist();

    // Sync folder watches
    if let Ok(mut w) = state.watcher.lock() {
        if let Some(watcher) = w.as_mut() {
            watcher.sync_watched_folders(&folders);
        }
    }
}

#[tauri::command]
pub fn remove_folder(state: State<'_, AppState>, path: String) {
    let mut settings = state.settings.lock().unwrap();
    settings.folders.retain(|f| f != &path);
    if settings.active_folder.as_deref() == Some(&path) {
        settings.active_folder = None;
    }
    let folders = settings.folders.clone();
    drop(settings);
    state.persist();

    // Evict cache entries under the removed folder
    let all = state.cache.all_paths();
    let remaining: Vec<String> = all
        .into_iter()
        .filter(|p| !p.starts_with(&path))
        .collect();
    state.cache.evict_not_in(&remaining);

    // Sync watcher — both repo watches and folder watches
    if let Ok(mut w) = state.watcher.lock() {
        if let Some(watcher) = w.as_mut() {
            watcher.sync_watched_repos(&remaining);
            watcher.sync_watched_folders(&folders);
        }
    }
}

#[tauri::command]
pub fn set_active_folder(state: State<'_, AppState>, path: Option<String>) {
    let mut settings = state.settings.lock().unwrap();
    settings.active_folder = path;
    drop(settings);
    state.persist();
}

#[tauri::command]
pub fn scan_repos(state: State<'_, AppState>, roots: Vec<String>) -> Vec<RepoSummary> {
    let settings = state.settings.lock().unwrap().clone();

    let repo_paths = scanner::discover_repos(&roots, &settings.discovery_exclusions);

    let summaries: Vec<RepoSummary> = repo_paths
        .iter()
        .map(|repo_path| {
            let mut summary =
                git_ops::build_repo_summary(repo_path, settings.include_untracked_mtime);

            // Check for high-risk ignored files
            summary.has_high_risk_ignored = risk::has_high_risk_files(
                repo_path,
                &settings.always_flag_patterns,
                &settings.always_ignore_patterns,
            );

            summary
        })
        .collect();

    summaries
}

#[tauri::command]
pub fn get_repo_detail(
    state: State<'_, AppState>,
    repo_path: String,
) -> Result<RepoDetail, String> {
    let settings = state.settings.lock().unwrap().clone();
    let path = Path::new(&repo_path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", repo_path));
    }

    let summary = git_ops::build_repo_summary(path, settings.include_untracked_mtime);
    let branches = git_ops::get_branches(path);
    let ignored_files = risk::find_all_ignored_files(
        path,
        &settings.always_flag_patterns,
        &settings.always_ignore_patterns,
    );
    let changed_files = git_ops::get_changed_files(path);
    let recent_commits = git_ops::get_recent_commits(path, 10);
    let warnings = git_ops::get_warnings(path, &summary.sync_status, &summary.remotes);

    Ok(RepoDetail {
        summary,
        branches,
        ignored_files,
        changed_files,
        recent_commits,
        warnings,
    })
}

#[tauri::command]
pub fn refresh_repo(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_path: String,
) -> Result<RepoSummary, String> {
    let settings = state.settings.lock().unwrap().clone();
    let path = Path::new(&repo_path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", repo_path));
    }

    let mut summary = git_ops::build_repo_summary(path, settings.include_untracked_mtime);
    summary.has_high_risk_ignored = risk::has_high_risk_files(
        path,
        &settings.always_flag_patterns,
        &settings.always_ignore_patterns,
    );

    // Update cache
    let fingerprint = RepoCache::compute_fingerprint(path);
    state
        .cache
        .update(&repo_path, summary.clone(), fingerprint);

    // Emit update event for the frontend
    let _ = app.emit("scan:repo-updated", &summary);

    Ok(summary)
}

#[tauri::command]
pub fn start_scan(app: AppHandle, state: State<'_, AppState>, roots: Vec<String>) {
    let settings = state.settings.lock().unwrap().clone();
    let scan_id = state.current_scan_id.fetch_add(1, Ordering::SeqCst) + 1;
    let current_scan_id = Arc::clone(&state.current_scan_id);
    let cache = Arc::clone(&state.cache);
    let watcher = Arc::clone(&state.watcher);

    std::thread::spawn(move || {
        // Phase 1: Discovery
        let repo_paths = scanner::discover_repos(&roots, &settings.discovery_exclusions);

        if current_scan_id.load(Ordering::SeqCst) != scan_id {
            return;
        }

        let total = repo_paths.len();
        let _ = app.emit(
            "scan:discovery-complete",
            ScanDiscoveryComplete { total },
        );

        // Phase 2: Analysis (with cache)
        let mut repo_path_strings = Vec::with_capacity(total);

        for (i, repo_path) in repo_paths.iter().enumerate() {
            if current_scan_id.load(Ordering::SeqCst) != scan_id {
                return;
            }

            let path_str = repo_path.to_string_lossy().to_string();
            repo_path_strings.push(path_str.clone());

            let repo_name = repo_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let _ = app.emit(
                "scan:progress",
                ScanProgress {
                    total,
                    completed: i,
                    current_repo: Some(repo_name),
                },
            );

            // Fingerprint check
            let fingerprint = RepoCache::compute_fingerprint(repo_path);

            if !cache.is_stale(&path_str, &fingerprint) {
                // Cache hit — emit cached summary
                if let Some(summary) = cache.get(&path_str) {
                    if current_scan_id.load(Ordering::SeqCst) != scan_id {
                        return;
                    }
                    let _ = app.emit("scan:repo-ready", &summary);
                    continue;
                }
            }

            // Cache miss — full analysis
            let mut summary =
                git_ops::build_repo_summary(repo_path, settings.include_untracked_mtime);
            summary.has_high_risk_ignored = risk::has_high_risk_files(
                repo_path,
                &settings.always_flag_patterns,
                &settings.always_ignore_patterns,
            );

            // Update cache
            cache.update(&path_str, summary.clone(), fingerprint);

            if current_scan_id.load(Ordering::SeqCst) != scan_id {
                return;
            }

            let _ = app.emit("scan:repo-ready", &summary);
        }

        // Evict repos that are no longer discovered
        cache.evict_not_in(&repo_path_strings);

        // Sync file watcher to the discovered repo set
        if let Ok(mut w) = watcher.lock() {
            if let Some(watcher) = w.as_mut() {
                watcher.sync_watched_repos(&repo_path_strings);
            }
        }

        if current_scan_id.load(Ordering::SeqCst) != scan_id {
            return;
        }

        let _ = app.emit("scan:complete", ());
    });
}

/// Rescan a single repo and emit an update event. Used by watcher and poller.
/// If the repo no longer exists, evicts from cache and emits a removal event.
pub fn rescan_single_repo_and_emit(
    app: &AppHandle,
    cache: &RepoCache,
    repo_path: &str,
    settings: &AppSettings,
) {
    let path = Path::new(repo_path);
    if !path.exists() {
        cache.evict(repo_path);
        let _ = app.emit(
            "scan:repo-removed",
            RepoRemoved {
                path: repo_path.to_string(),
            },
        );
        return;
    }

    let fingerprint = RepoCache::compute_fingerprint(path);
    if !cache.is_stale(repo_path, &fingerprint) {
        return; // Still fresh
    }

    let mut summary = git_ops::build_repo_summary(path, settings.include_untracked_mtime);
    summary.has_high_risk_ignored = risk::has_high_risk_files(
        path,
        &settings.always_flag_patterns,
        &settings.always_ignore_patterns,
    );

    cache.update(repo_path, summary.clone(), fingerprint);
    let _ = app.emit("scan:repo-updated", &summary);
}

/// Reconcile discovered repos with the cache: add new repos, remove stale ones.
/// Called by the poller on a timer and when the discovery_needed flag fires.
pub fn reconcile_repos(
    app: &AppHandle,
    cache: &RepoCache,
    settings: &AppSettings,
    watcher: &Arc<Mutex<Option<crate::watcher::GitWatcher>>>,
) {
    let discovered = scanner::discover_repos(&settings.folders, &settings.discovery_exclusions);
    let discovered_set: std::collections::HashSet<String> = discovered
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let cached_set: std::collections::HashSet<String> = cache.all_paths().into_iter().collect();

    // New repos: discovered but not cached
    for repo_path in &discovered {
        let path_str = repo_path.to_string_lossy().to_string();
        if !cached_set.contains(&path_str) {
            let mut summary =
                git_ops::build_repo_summary(repo_path, settings.include_untracked_mtime);
            summary.has_high_risk_ignored = risk::has_high_risk_files(
                repo_path,
                &settings.always_flag_patterns,
                &settings.always_ignore_patterns,
            );
            let fingerprint = RepoCache::compute_fingerprint(repo_path);
            cache.update(&path_str, summary.clone(), fingerprint);
            let _ = app.emit("scan:repo-updated", &summary);
        }
    }

    // Removed repos: cached but not discovered
    for path_str in &cached_set {
        if !discovered_set.contains(path_str) {
            cache.evict(path_str);
            let _ = app.emit(
                "scan:repo-removed",
                RepoRemoved {
                    path: path_str.clone(),
                },
            );
        }
    }

    // Sync watcher repo watches
    let all_paths: Vec<String> = discovered_set.into_iter().collect();
    if let Ok(mut w) = watcher.lock() {
        if let Some(watcher) = w.as_mut() {
            watcher.sync_watched_repos(&all_paths);
        }
    }
}

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use tauri::AppHandle;

use crate::cache::RepoCache;
use crate::commands::rescan_single_repo_and_emit;
use crate::models::AppSettings;

type NotifyWatcher = notify::RecommendedWatcher;

/// Maps watched path (`.git/` dir or repo root) -> repo root path string.
type RepoMap = Arc<Mutex<HashMap<PathBuf, String>>>;

/// Maps watched scan-root folder -> folder path string.
type FolderMap = Arc<Mutex<HashMap<PathBuf, String>>>;

pub struct GitWatcher {
    debouncer: Debouncer<NotifyWatcher>,
    watched_repos: HashSet<String>,
    repo_map: RepoMap,
    watched_folders: HashSet<String>,
    folder_map: FolderMap,
}

impl GitWatcher {
    /// Create a new GitWatcher that watches .git/ directories, repo roots, and scan folders.
    pub fn new(
        app: AppHandle,
        cache: Arc<RepoCache>,
        settings_mutex: Arc<Mutex<AppSettings>>,
        discovery_needed: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let repo_map: RepoMap = Arc::new(Mutex::new(HashMap::new()));
        let repo_map_for_handler = Arc::clone(&repo_map);

        let folder_map: FolderMap = Arc::new(Mutex::new(HashMap::new()));
        let folder_map_for_handler = Arc::clone(&folder_map);

        let discovery_needed_for_handler = Arc::clone(&discovery_needed);

        let debouncer = new_debouncer(
            Duration::from_millis(500),
            move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
                let events = match events {
                    Ok(e) => e,
                    Err(_) => return,
                };

                // Collect unique repo roots that need rescanning
                let mut repos_to_rescan: HashSet<String> = HashSet::new();
                let mut folder_event = false;

                let rmap = repo_map_for_handler.lock().unwrap();
                let fmap = folder_map_for_handler.lock().unwrap();

                for event in &events {
                    if event.kind != DebouncedEventKind::Any {
                        continue;
                    }
                    let changed = &event.path;

                    // First check repo_map — events inside known repos trigger rescan
                    let mut matched_repo = false;
                    for (watched_path, repo_root) in rmap.iter() {
                        if changed.starts_with(watched_path) {
                            repos_to_rescan.insert(repo_root.clone());
                            matched_repo = true;
                            break;
                        }
                    }

                    // If not a known repo event, check folder_map — signals discovery needed
                    if !matched_repo {
                        for (watched_path, _) in fmap.iter() {
                            if changed.starts_with(watched_path) {
                                folder_event = true;
                                break;
                            }
                        }
                    }
                }
                drop(rmap);
                drop(fmap);

                // Rescan each affected repo
                for repo_path in repos_to_rescan {
                    let settings = settings_mutex.lock().unwrap().clone();
                    rescan_single_repo_and_emit(&app, &cache, &repo_path, &settings);
                }

                // Signal discovery needed for folder events outside known repos
                if folder_event {
                    discovery_needed_for_handler.store(true, Ordering::Release);
                }
            },
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        Ok(Self {
            debouncer,
            watched_repos: HashSet::new(),
            repo_map,
            watched_folders: HashSet::new(),
            folder_map,
        })
    }

    /// Start watching a repo's .git directory and repo root.
    pub fn watch(&mut self, repo_path: &str) -> Result<(), String> {
        let git_dir = resolve_watch_path(Path::new(repo_path));
        if !git_dir.exists() {
            return Err(format!(".git dir not found for {}", repo_path));
        }

        self.debouncer
            .watcher()
            .watch(&git_dir, RecursiveMode::Recursive)
            .map_err(|e| format!("Watch failed for {}: {}", repo_path, e))?;

        // Also watch the repo root recursively for working-tree edits.
        // On macOS (FSEvents) this is essentially free. On Linux (inotify) this
        // can fail for large trees — if so, the .git watch + poller still cover it.
        let root = PathBuf::from(repo_path);
        if let Err(e) = self
            .debouncer
            .watcher()
            .watch(&root, RecursiveMode::Recursive)
        {
            eprintln!(
                "Warning: could not watch repo root {} (poller will cover it): {}",
                repo_path, e
            );
        }

        // Register mappings so events from either path route back to the repo root
        {
            let mut map = self.repo_map.lock().unwrap();
            map.insert(git_dir, repo_path.to_string());
            map.insert(root, repo_path.to_string());
        }

        self.watched_repos.insert(repo_path.to_string());
        Ok(())
    }

    /// Stop watching a repo.
    pub fn unwatch(&mut self, repo_path: &str) {
        let git_dir = resolve_watch_path(Path::new(repo_path));
        let root = PathBuf::from(repo_path);
        let _ = self.debouncer.watcher().unwatch(&git_dir);
        let _ = self.debouncer.watcher().unwatch(&root);

        {
            let mut map = self.repo_map.lock().unwrap();
            map.remove(&git_dir);
            map.remove(&root);
        }

        self.watched_repos.remove(repo_path);
    }

    /// Sync the watched set to match the current set of discovered repos.
    pub fn sync_watched_repos(&mut self, current_repos: &[String]) {
        let current_set: HashSet<&String> = current_repos.iter().collect();
        let watched_set: HashSet<String> = self.watched_repos.clone();

        // Remove watches for repos no longer present
        for repo in &watched_set {
            if !current_set.contains(repo) {
                self.unwatch(repo);
            }
        }

        // Add watches for new repos
        for repo in current_repos {
            if !self.watched_repos.contains(repo) {
                if let Err(e) = self.watch(repo) {
                    eprintln!("Warning: could not watch {}: {}", repo, e);
                }
            }
        }
    }

    /// Start watching a scan root folder for new/removed repos.
    pub fn watch_folder(&mut self, folder_path: &str) -> Result<(), String> {
        let path = PathBuf::from(folder_path);
        if !path.exists() || !path.is_dir() {
            return Err(format!("Folder does not exist: {}", folder_path));
        }

        // Watch recursively — on macOS (FSEvents) this is cheap (single stream).
        // On Linux (inotify) this may fail for huge trees; that's OK, poller covers it.
        if let Err(e) = self
            .debouncer
            .watcher()
            .watch(&path, RecursiveMode::Recursive)
        {
            eprintln!(
                "Warning: could not watch folder {} (poller will cover it): {}",
                folder_path, e
            );
            return Ok(()); // Non-fatal
        }

        {
            let mut map = self.folder_map.lock().unwrap();
            map.insert(path, folder_path.to_string());
        }

        self.watched_folders.insert(folder_path.to_string());
        Ok(())
    }

    /// Stop watching a scan root folder.
    pub fn unwatch_folder(&mut self, folder_path: &str) {
        let path = PathBuf::from(folder_path);
        let _ = self.debouncer.watcher().unwatch(&path);

        {
            let mut map = self.folder_map.lock().unwrap();
            map.remove(&path);
        }

        self.watched_folders.remove(folder_path);
    }

    /// Sync watched folders to match the current settings.
    pub fn sync_watched_folders(&mut self, current_folders: &[String]) {
        let current_set: HashSet<&String> = current_folders.iter().collect();
        let watched_set: HashSet<String> = self.watched_folders.clone();

        // Remove watches for folders no longer configured
        for folder in &watched_set {
            if !current_set.contains(folder) {
                self.unwatch_folder(folder);
            }
        }

        // Add watches for new folders
        for folder in current_folders {
            if !self.watched_folders.contains(folder) {
                if let Err(e) = self.watch_folder(folder) {
                    eprintln!("Warning: could not watch folder {}: {}", folder, e);
                }
            }
        }
    }
}

/// Resolve the path to watch — handles .git files (worktrees/submodules).
fn resolve_watch_path(repo_path: &Path) -> PathBuf {
    let dot_git = repo_path.join(".git");
    if dot_git.is_file() {
        if let Ok(content) = std::fs::read_to_string(&dot_git) {
            let trimmed = content.trim();
            if let Some(gitdir) = trimmed.strip_prefix("gitdir: ") {
                let gitdir_path = PathBuf::from(gitdir);
                if gitdir_path.is_absolute() {
                    return gitdir_path;
                } else {
                    return repo_path.join(gitdir_path);
                }
            }
        }
    }
    dot_git
}

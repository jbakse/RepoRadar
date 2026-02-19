use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::AppHandle;

use crate::cache::RepoCache;
use crate::commands::{reconcile_repos, rescan_single_repo_and_emit};
use crate::models::AppSettings;
use crate::watcher::GitWatcher;

const POLL_INTERVAL_SECS: u64 = 120;
const TICK_MS: u64 = 100;
const DISCOVERY_DEBOUNCE_SECS: u64 = 3;

pub struct WorkTreePoller {
    stop_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl WorkTreePoller {
    /// Start a background poller that checks cached repos for staleness and
    /// runs reconciliation when discovery_needed is flagged or every 120s.
    pub fn new(
        app: AppHandle,
        cache: Arc<RepoCache>,
        settings_mutex: Arc<Mutex<AppSettings>>,
        watcher: Arc<Mutex<Option<GitWatcher>>>,
        discovery_needed: Arc<AtomicBool>,
    ) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop_flag);

        let handle = thread::spawn(move || {
            let ticks_per_cycle = (POLL_INTERVAL_SECS * 1000) / TICK_MS;
            let debounce_ticks = (DISCOVERY_DEBOUNCE_SECS * 1000) / TICK_MS;
            let mut tick_count: u64 = 0;
            // Countdown for discovery debounce: None = idle, Some(n) = ticks remaining
            let mut discovery_countdown: Option<u64> = None;

            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    return;
                }

                thread::sleep(Duration::from_millis(TICK_MS));
                tick_count += 1;

                // Check discovery_needed flag
                if discovery_needed.swap(false, Ordering::AcqRel) {
                    // (Re)start the debounce countdown
                    discovery_countdown = Some(debounce_ticks);
                }

                // Tick down the discovery countdown
                if let Some(remaining) = discovery_countdown {
                    if remaining <= 1 {
                        // Debounce expired — run reconciliation
                        discovery_countdown = None;
                        let settings = settings_mutex.lock().unwrap().clone();
                        reconcile_repos(&app, &cache, &settings, &watcher);
                    } else {
                        discovery_countdown = Some(remaining - 1);
                    }
                }

                // Periodic cycle: staleness checks + reconciliation
                if tick_count >= ticks_per_cycle {
                    tick_count = 0;

                    if stop_clone.load(Ordering::Relaxed) {
                        return;
                    }

                    let settings = settings_mutex.lock().unwrap().clone();

                    // Check all cached repos for staleness
                    let paths = cache.all_paths();
                    for repo_path in paths {
                        if stop_clone.load(Ordering::Relaxed) {
                            return;
                        }
                        rescan_single_repo_and_emit(&app, &cache, &repo_path, &settings);
                    }

                    // Also run reconciliation to catch new/removed repos
                    reconcile_repos(&app, &cache, &settings, &watcher);
                }
            }
        });

        Self {
            stop_flag,
            handle: Some(handle),
        }
    }

    /// Signal the poller to stop.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

impl Drop for WorkTreePoller {
    fn drop(&mut self) {
        self.stop();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

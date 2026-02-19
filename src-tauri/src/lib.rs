mod cache;
mod commands;
mod git_ops;
mod models;
mod persistence;
mod poller;
mod risk;
mod scanner;
mod watcher;

use cache::RepoCache;
use commands::AppState;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let settings = persistence::load_settings(&app_data_dir);
            let settings_arc = Arc::new(Mutex::new(settings.clone()));
            let cache = Arc::new(RepoCache::new());

            // Shared flag: watcher sets this when folder events occur outside known repos
            let discovery_needed = Arc::new(AtomicBool::new(false));

            // Initialize file watcher (graceful degradation if it fails)
            let watcher = {
                let app_handle = app.handle().clone();
                match watcher::GitWatcher::new(
                    app_handle,
                    Arc::clone(&cache),
                    Arc::clone(&settings_arc),
                    Arc::clone(&discovery_needed),
                ) {
                    Ok(mut w) => {
                        // Start watching configured scan root folders
                        w.sync_watched_folders(&settings.folders);
                        eprintln!("File watcher initialized");
                        Some(w)
                    }
                    Err(e) => {
                        eprintln!("Warning: File watcher init failed: {}. Poller will cover.", e);
                        None
                    }
                }
            };

            let watcher_arc = Arc::new(Mutex::new(watcher));

            // Initialize poller
            let poller = {
                let app_handle = app.handle().clone();
                poller::WorkTreePoller::new(
                    app_handle,
                    Arc::clone(&cache),
                    Arc::clone(&settings_arc),
                    Arc::clone(&watcher_arc),
                    Arc::clone(&discovery_needed),
                )
            };

            app.manage(AppState {
                settings: Arc::clone(&settings_arc),
                config_dir: app_data_dir,
                current_scan_id: Arc::new(AtomicU64::new(0)),
                cache,
                watcher: watcher_arc,
                poller: Arc::new(Mutex::new(Some(poller))),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::add_folder,
            commands::remove_folder,
            commands::set_active_folder,
            commands::scan_repos,
            commands::start_scan,
            commands::get_repo_detail,
            commands::refresh_repo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

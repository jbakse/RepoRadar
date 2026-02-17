mod commands;
mod git_ops;
mod models;
mod persistence;
mod risk;
mod scanner;

use commands::AppState;
use std::sync::atomic::AtomicU64;
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

            app.manage(AppState {
                settings: Mutex::new(settings),
                config_dir: app_data_dir,
                current_scan_id: Arc::new(AtomicU64::new(0)),
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

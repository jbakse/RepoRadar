mod commands;
mod git_ops;
mod models;
mod risk;
mod scanner;

use commands::AppState;
use models::AppSettings;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            settings: Mutex::new(AppSettings::default()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::add_workspace,
            commands::remove_workspace,
            commands::scan_repos,
            commands::get_repo_detail,
            commands::refresh_repo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

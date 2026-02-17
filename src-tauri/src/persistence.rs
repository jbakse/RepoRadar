use std::fs;
use std::path::PathBuf;

use crate::models::AppSettings;

/// Get the settings file path within the given app data directory.
pub fn settings_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("settings.json")
}

/// Load settings from disk, falling back to defaults if file doesn't exist or is invalid.
/// Migrates legacy workspace data to the flat folders model.
pub fn load_settings(app_data_dir: &PathBuf) -> AppSettings {
    let path = settings_path(app_data_dir);
    let mut settings: AppSettings = match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => return AppSettings::default(),
    };

    // Migrate legacy workspaces → flat folders
    if settings.folders.is_empty() && !settings.workspaces.is_empty() {
        let mut all_roots: Vec<String> = settings
            .workspaces
            .iter()
            .flat_map(|w| w.roots.clone())
            .collect();
        all_roots.sort();
        all_roots.dedup();
        settings.folders = all_roots;
        settings.workspaces.clear();
        settings.last_active_workspace = None;
        // Persist the migrated settings
        save_settings(app_data_dir, &settings);
    }

    settings
}

/// Save settings to disk. Creates the directory if needed.
pub fn save_settings(app_data_dir: &PathBuf, settings: &AppSettings) {
    if let Err(e) = fs::create_dir_all(app_data_dir) {
        eprintln!("Failed to create config dir: {}", e);
        return;
    }
    let path = settings_path(app_data_dir);
    match serde_json::to_string_pretty(settings) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("Failed to write settings: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to serialize settings: {}", e),
    }
}

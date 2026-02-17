use std::fs;
use std::path::PathBuf;

use crate::models::AppSettings;

/// Get the settings file path within the given app data directory.
pub fn settings_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("settings.json")
}

/// Load settings from disk, falling back to defaults if file doesn't exist or is invalid.
pub fn load_settings(app_data_dir: &PathBuf) -> AppSettings {
    let path = settings_path(app_data_dir);
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
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

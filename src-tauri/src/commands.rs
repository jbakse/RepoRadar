use std::path::Path;
use std::sync::Mutex;

use tauri::State;

use crate::git_ops;
use crate::models::*;
use crate::risk;
use crate::scanner;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) {
    let mut s = state.settings.lock().unwrap();
    *s = settings;
}

#[tauri::command]
pub fn add_workspace(state: State<'_, AppState>, name: String, roots: Vec<String>) {
    let mut settings = state.settings.lock().unwrap();
    // Remove existing workspace with same name
    settings.workspaces.retain(|w| w.name != name);
    settings.workspaces.push(Workspace { name, roots });
}

#[tauri::command]
pub fn remove_workspace(state: State<'_, AppState>, name: String) {
    let mut settings = state.settings.lock().unwrap();
    settings.workspaces.retain(|w| w.name != name);
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
pub fn get_repo_detail(state: State<'_, AppState>, repo_path: String) -> Result<RepoDetail, String> {
    let settings = state.settings.lock().unwrap().clone();
    let path = Path::new(&repo_path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", repo_path));
    }

    let summary = git_ops::build_repo_summary(path, settings.include_untracked_mtime);
    let branches = git_ops::get_branches(path);
    let ignored_files = risk::find_ignored_files(
        path,
        &settings.always_flag_patterns,
        &settings.always_ignore_patterns,
    );
    let (staged_files, unstaged_files, untracked_files) = git_ops::get_file_lists(path);
    let warnings = git_ops::get_warnings(path, &summary.sync_status, &summary.remotes);

    Ok(RepoDetail {
        summary,
        branches,
        ignored_files,
        staged_files,
        unstaged_files,
        untracked_files,
        warnings,
    })
}

#[tauri::command]
pub fn refresh_repo(state: State<'_, AppState>, repo_path: String) -> Result<RepoSummary, String> {
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

    Ok(summary)
}

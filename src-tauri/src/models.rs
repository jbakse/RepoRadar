use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSummary {
    pub path: String,
    pub folder_name: String,
    pub repo_name: String,
    pub created: Option<String>,
    pub last_commit: Option<CommitInfo>,
    pub last_file_edited: Option<FileEditInfo>,
    pub uncommitted: UncommittedChanges,
    pub sync_status: SyncStatus,
    pub remotes: Vec<RemoteInfo>,
    pub has_high_risk_ignored: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub subject: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditInfo {
    pub path: String,
    pub mtime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UncommittedChanges {
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStatus {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub detached: bool,
    pub branches_without_upstream: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    pub name: String,
    pub fetch_url: String,
    pub push_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoredFileInfo {
    pub path: String,
    pub risk_level: RiskLevel,
    pub matched_rule: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDetail {
    pub summary: RepoSummary,
    pub branches: Vec<BranchInfo>,
    pub ignored_files: Vec<IgnoredFileInfo>,
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total: usize,
    pub completed: usize,
    pub current_repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDiscoveryComplete {
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub workspaces: Vec<Workspace>,
    pub discovery_exclusions: Vec<String>,
    pub always_flag_patterns: Vec<String>,
    pub always_ignore_patterns: Vec<String>,
    pub include_untracked_mtime: bool,
    #[serde(default)]
    pub last_active_workspace: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            workspaces: vec![],
            discovery_exclusions: vec![
                "node_modules".to_string(),
                ".venv".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
            ],
            always_flag_patterns: vec![
                ".env".to_string(),
                ".env.*".to_string(),
                "*.key".to_string(),
                "*.pem".to_string(),
                "*.p12".to_string(),
                "id_rsa*".to_string(),
                "*.mobileprovision".to_string(),
            ],
            always_ignore_patterns: vec![
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                ".DS_Store".to_string(),
                ".next/**".to_string(),
                "__pycache__/**".to_string(),
                "*.pyc".to_string(),
                "target/**".to_string(),
            ],
            include_untracked_mtime: false,
            last_active_workspace: None,
        }
    }
}

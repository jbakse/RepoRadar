export interface RepoSummary {
  path: string;
  folder_name: string;
  repo_name: string;
  created: string | null;
  last_commit: CommitInfo | null;
  last_file_edited: FileEditInfo | null;
  uncommitted: UncommittedChanges;
  sync_status: SyncStatus;
  remotes: RemoteInfo[];
  has_high_risk_ignored: boolean;
  error: string | null;
}

export interface CommitInfo {
  hash: string;
  subject: string;
  author: string;
  timestamp: string;
}

export interface FileEditInfo {
  path: string;
  mtime: string;
}

export interface UncommittedChanges {
  staged: number;
  unstaged: number;
  untracked: number;
}

export interface SyncStatus {
  branch: string | null;
  upstream: string | null;
  ahead: number;
  behind: number;
  detached: boolean;
  branches_without_upstream: string[];
}

export interface RemoteInfo {
  name: string;
  fetch_url: string;
  push_url: string;
}

export interface BranchInfo {
  name: string;
  upstream: string | null;
  ahead: number;
  behind: number;
  is_current: boolean;
}

export interface IgnoredFileInfo {
  path: string;
  risk_level: "High" | "Medium" | "Low";
  matched_rule: string;
  category: string;
  is_directory: boolean;
  child_count: number;
}

export type FileStatus =
  | "Modified"
  | "Added"
  | "Deleted"
  | "Renamed"
  | "Copied"
  | "Untracked"
  | "TypeChanged";

export interface ChangedFileEntry {
  path: string;
  file_name: string;
  status: FileStatus;
  staged: boolean;
}

export interface RepoDetail {
  summary: RepoSummary;
  branches: BranchInfo[];
  ignored_files: IgnoredFileInfo[];
  changed_files: ChangedFileEntry[];
  recent_commits: CommitInfo[];
  warnings: string[];
}

export interface AppSettings {
  folders: string[];
  active_folder: string | null;
  discovery_exclusions: string[];
  always_flag_patterns: string[];
  always_ignore_patterns: string[];
  include_untracked_mtime: boolean;
}

export type SortField =
  | "name"
  | "created"
  | "last_commit"
  | "last_edited"
  | "changes"
  | "sync"
  | "risk";

export type SortDirection = "asc" | "desc";

export type FilterType =
  | "all"
  | "dirty"
  | "unpushed"
  | "missing_remote"
  | "high_risk";

export interface ScanProgress {
  total: number;
  completed: number;
  current_repo: string | null;
}

export interface ScanDiscoveryComplete {
  total: number;
}

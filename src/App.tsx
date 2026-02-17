import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  RepoSummary,
  RepoDetail,
  Workspace,
  AppSettings,
  FilterType,
  SortField,
  SortDirection,
} from "./types";
import { RepoTable } from "./components/RepoTable";
import { RepoDetailView } from "./components/RepoDetailView";
import { WorkspacePicker } from "./components/WorkspacePicker";
import { FilterBar } from "./components/FilterBar";
import { SettingsModal } from "./components/SettingsModal";
import "./App.css";

function App() {
  const [repos, setRepos] = useState<RepoSummary[]>([]);
  const [selectedRepo, setSelectedRepo] = useState<RepoDetail | null>(null);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [activeWorkspace, setActiveWorkspace] = useState<Workspace | null>(
    null
  );
  const [scanning, setScanning] = useState(false);
  const [filter, setFilter] = useState<FilterType>("all");
  const [sortField, setSortField] = useState<SortField>("name");
  const [sortDir, setSortDir] = useState<SortDirection>("asc");
  const [view, setView] = useState<"table" | "detail">("table");
  const [showSettings, setShowSettings] = useState(false);

  const scanRoots = useCallback(async (roots: string[]) => {
    if (roots.length === 0) return;
    setScanning(true);
    setSelectedRepo(null);
    setView("table");
    try {
      const results = await invoke<RepoSummary[]>("scan_repos", { roots });
      setRepos(results);
    } catch (err) {
      console.error("Scan failed:", err);
    } finally {
      setScanning(false);
    }
  }, []);

  // Load saved settings on mount and restore last workspace
  useEffect(() => {
    async function loadSettings() {
      try {
        const settings = await invoke<AppSettings>("get_settings");
        setWorkspaces(settings.workspaces);
        if (settings.last_active_workspace) {
          const ws = settings.workspaces.find(
            (w) => w.name === settings.last_active_workspace
          );
          if (ws) {
            setActiveWorkspace(ws);
            scanRoots(ws.roots);
          }
        }
      } catch (err) {
        console.error("Failed to load settings:", err);
      }
    }
    loadSettings();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleSelectWorkspace = useCallback(
    (workspace: Workspace) => {
      setActiveWorkspace(workspace);
      invoke("set_active_workspace", { name: workspace.name });
      scanRoots(workspace.roots);
    },
    [scanRoots]
  );

  const handleAddFolder = useCallback(async () => {
    const selected = await open({ directory: true, multiple: true });
    if (selected) {
      const folders = Array.isArray(selected) ? selected : [selected];
      if (activeWorkspace) {
        const newRoots = [...new Set([...activeWorkspace.roots, ...folders])];
        const updated = { ...activeWorkspace, roots: newRoots };
        setActiveWorkspace(updated);
        setWorkspaces((prev) =>
          prev.map((w) => (w.name === updated.name ? updated : w))
        );
        await invoke("add_workspace", {
          name: updated.name,
          roots: updated.roots,
        });
        scanRoots(newRoots);
      } else {
        const name = "Default";
        const ws: Workspace = { name, roots: folders };
        setWorkspaces((prev) => [...prev, ws]);
        setActiveWorkspace(ws);
        await invoke("add_workspace", { name, roots: folders });
        scanRoots(folders);
      }
    }
  }, [activeWorkspace, scanRoots]);

  const handleCreateWorkspace = useCallback(
    async (name: string) => {
      const selected = await open({ directory: true, multiple: true });
      if (selected) {
        const folders = Array.isArray(selected) ? selected : [selected];
        const ws: Workspace = { name, roots: folders };
        setWorkspaces((prev) => [...prev.filter((w) => w.name !== name), ws]);
        setActiveWorkspace(ws);
        await invoke("add_workspace", { name, roots: folders });
        scanRoots(folders);
      }
    },
    [scanRoots]
  );

  const handleDeleteWorkspace = useCallback(
    async (name: string) => {
      setWorkspaces((prev) => prev.filter((w) => w.name !== name));
      if (activeWorkspace?.name === name) {
        setActiveWorkspace(null);
        setRepos([]);
      }
      await invoke("remove_workspace", { name });
    },
    [activeWorkspace]
  );

  const handleRefresh = useCallback(() => {
    if (activeWorkspace) {
      scanRoots(activeWorkspace.roots);
    }
  }, [activeWorkspace, scanRoots]);

  const handleSelectRepo = useCallback(async (repo: RepoSummary) => {
    try {
      const detail = await invoke<RepoDetail>("get_repo_detail", {
        repoPath: repo.path,
      });
      setSelectedRepo(detail);
      setView("detail");
    } catch (err) {
      console.error("Failed to load repo detail:", err);
    }
  }, []);

  const handleBackToTable = useCallback(() => {
    setView("table");
    setSelectedRepo(null);
  }, []);

  const handleSaveSettings = useCallback(
    async (settings: AppSettings) => {
      await invoke("save_settings", { settings });
      setWorkspaces(settings.workspaces);
      // Re-scan if active workspace still exists (settings may have changed scan behavior)
      if (activeWorkspace) {
        const ws = settings.workspaces.find(
          (w) => w.name === activeWorkspace.name
        );
        if (ws) {
          scanRoots(ws.roots);
        }
      }
    },
    [activeWorkspace, scanRoots]
  );

  const handleSort = useCallback(
    (field: SortField) => {
      if (sortField === field) {
        setSortDir((d) => (d === "asc" ? "desc" : "asc"));
      } else {
        setSortField(field);
        setSortDir("asc");
      }
    },
    [sortField]
  );

  const filteredRepos = repos.filter((repo) => {
    switch (filter) {
      case "dirty":
        return (
          repo.uncommitted.staged > 0 ||
          repo.uncommitted.unstaged > 0 ||
          repo.uncommitted.untracked > 0
        );
      case "unpushed":
        return repo.sync_status.ahead > 0;
      case "missing_remote":
        return repo.remotes.length === 0;
      case "high_risk":
        return repo.has_high_risk_ignored;
      default:
        return true;
    }
  });

  const sortedRepos = [...filteredRepos].sort((a, b) => {
    const dir = sortDir === "asc" ? 1 : -1;
    switch (sortField) {
      case "name":
        return dir * a.repo_name.localeCompare(b.repo_name);
      case "created":
        return dir * ((a.created || "") > (b.created || "") ? 1 : -1);
      case "last_commit":
        return (
          dir *
          ((a.last_commit?.timestamp || "") >
          (b.last_commit?.timestamp || "")
            ? 1
            : -1)
        );
      case "last_edited":
        return (
          dir *
          ((a.last_file_edited?.mtime || "") >
          (b.last_file_edited?.mtime || "")
            ? 1
            : -1)
        );
      case "changes": {
        const totalA =
          a.uncommitted.staged +
          a.uncommitted.unstaged +
          a.uncommitted.untracked;
        const totalB =
          b.uncommitted.staged +
          b.uncommitted.unstaged +
          b.uncommitted.untracked;
        return dir * (totalA - totalB);
      }
      case "sync":
        return dir * (a.sync_status.ahead - b.sync_status.ahead);
      case "risk":
        return (
          dir *
          (Number(b.has_high_risk_ignored) - Number(a.has_high_risk_ignored))
        );
      default:
        return 0;
    }
  });

  return (
    <div className="app">
      <header className="app-header">
        <h1 className="app-title">RepoRadar</h1>
        <div className="header-actions">
          <WorkspacePicker
            workspaces={workspaces}
            activeWorkspace={activeWorkspace}
            onSelect={handleSelectWorkspace}
            onCreate={handleCreateWorkspace}
            onDelete={handleDeleteWorkspace}
          />
          <button className="btn btn-primary" onClick={handleAddFolder}>
            Add Folder
          </button>
          <button
            className="btn btn-secondary"
            onClick={handleRefresh}
            disabled={scanning || !activeWorkspace}
          >
            {scanning ? "Scanning..." : "Refresh"}
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => setShowSettings(true)}
            title="Settings"
          >
            Settings
          </button>
        </div>
      </header>

      <main className="app-main">
        {view === "table" ? (
          <>
            <FilterBar
              filter={filter}
              onFilterChange={setFilter}
              repoCount={repos.length}
              filteredCount={filteredRepos.length}
            />
            {repos.length === 0 && !scanning ? (
              <div className="empty-state">
                <div className="empty-state-icon">&#128269;</div>
                <h2>No repositories found</h2>
                <p>
                  Add a folder to scan for Git repositories, or create a
                  workspace to get started.
                </p>
                <button className="btn btn-primary" onClick={handleAddFolder}>
                  Add Folder
                </button>
              </div>
            ) : scanning ? (
              <div className="scanning-state">
                <div className="spinner"></div>
                <p>Scanning for repositories...</p>
              </div>
            ) : (
              <RepoTable
                repos={sortedRepos}
                sortField={sortField}
                sortDir={sortDir}
                onSort={handleSort}
                onSelect={handleSelectRepo}
              />
            )}
          </>
        ) : selectedRepo ? (
          <RepoDetailView detail={selectedRepo} onBack={handleBackToTable} />
        ) : null}
      </main>

      {showSettings && (
        <SettingsModal
          onClose={() => setShowSettings(false)}
          onSave={handleSaveSettings}
        />
      )}
    </div>
  );
}

export default App;

import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  RepoSummary,
  RepoDetail,
  AppSettings,
  FilterType,
  SortField,
  SortDirection,
  ScanProgress,
  ScanDiscoveryComplete,
} from "./types";
import { RepoTable } from "./components/RepoTable";
import { RepoDetailView } from "./components/RepoDetailView";
import { WorkspacePicker } from "./components/WorkspacePicker";
import { FilterBar } from "./components/FilterBar";
import { SettingsModal } from "./components/SettingsModal";
import { ScanProgressBar } from "./components/ScanProgressBar";
import "./App.css";

function App() {
  const [repos, setRepos] = useState<RepoSummary[]>([]);
  const [selectedRepo, setSelectedRepo] = useState<RepoDetail | null>(null);
  const [folders, setFolders] = useState<string[]>([]);
  const [activeFolder, setActiveFolder] = useState<string | null>(null);
  const [scanState, setScanState] = useState<{
    phase: "idle" | "discovering" | "analyzing";
    total: number;
    completed: number;
    currentRepo: string | null;
  }>({ phase: "idle", total: 0, completed: 0, currentRepo: null });
  const scanning = scanState.phase !== "idle";
  const [filter, setFilter] = useState<FilterType>("all");
  const [sortField, setSortField] = useState<SortField>("name");
  const [sortDir, setSortDir] = useState<SortDirection>("asc");
  const [view, setView] = useState<"table" | "detail">("table");
  const [initialTab, setInitialTab] = useState<string>("overview");
  const [showSettings, setShowSettings] = useState(false);

  const unlistenRef = useRef<UnlistenFn[]>([]);
  const initialLoadDone = useRef(false);

  const cleanupListeners = useCallback(() => {
    for (const unlisten of unlistenRef.current) {
      unlisten();
    }
    unlistenRef.current = [];
  }, []);

  // Clean up listeners on unmount
  useEffect(() => {
    return () => cleanupListeners();
  }, [cleanupListeners]);

  const scanRoots = useCallback(async (roots: string[]) => {
    if (roots.length === 0) return;

    // Clean up any existing listeners from a previous scan
    cleanupListeners();

    setRepos([]);
    setSelectedRepo(null);
    setView("table");
    setScanState({ phase: "discovering", total: 0, completed: 0, currentRepo: null });

    // Set up event listeners before starting the scan
    const unlistens = await Promise.all([
      listen<ScanDiscoveryComplete>("scan:discovery-complete", (event) => {
        setScanState((prev) => ({
          ...prev,
          phase: "analyzing",
          total: event.payload.total,
        }));
      }),
      listen<ScanProgress>("scan:progress", (event) => {
        setScanState((prev) => ({
          ...prev,
          completed: event.payload.completed,
          currentRepo: event.payload.current_repo,
        }));
      }),
      listen<RepoSummary>("scan:repo-ready", (event) => {
        setRepos((prev) => [...prev, event.payload]);
      }),
      listen("scan:complete", () => {
        setScanState({ phase: "idle", total: 0, completed: 0, currentRepo: null });
        // Defer cleanup so this handler can finish before being unregistered
        setTimeout(() => {
          for (const u of unlistenRef.current) u();
          unlistenRef.current = [];
        }, 0);
      }),
    ]);
    unlistenRef.current = unlistens;

    try {
      await invoke("start_scan", { roots });
    } catch (err) {
      console.error("Scan failed:", err);
      setScanState({ phase: "idle", total: 0, completed: 0, currentRepo: null });
      cleanupListeners();
    }
  }, [cleanupListeners]);

  // Helper: get roots to scan based on active folder
  const getRoots = useCallback(
    (folder: string | null, allFolders: string[]) => {
      return folder ? [folder] : allFolders;
    },
    []
  );

  // Load saved settings on mount and restore last active folder
  useEffect(() => {
    if (initialLoadDone.current) return;
    initialLoadDone.current = true;
    async function loadSettings() {
      try {
        const settings = await invoke<AppSettings>("get_settings");
        setFolders(settings.folders);
        setActiveFolder(settings.active_folder);
        const roots = settings.active_folder
          ? [settings.active_folder]
          : settings.folders;
        if (roots.length > 0) {
          scanRoots(roots);
        }
      } catch (err) {
        console.error("Failed to load settings:", err);
      }
    }
    loadSettings();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleSelectFolder = useCallback(
    (folder: string | null) => {
      setActiveFolder(folder);
      invoke("set_active_folder", { path: folder });
      scanRoots(getRoots(folder, folders));
    },
    [scanRoots, getRoots, folders]
  );

  const handleAddFolder = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected;
      const newFolders = folders.includes(path) ? folders : [...folders, path];
      setFolders(newFolders);
      setActiveFolder(path);
      await invoke("add_folder", { path });
      await invoke("set_active_folder", { path });
      scanRoots([path]);
    }
  }, [folders, scanRoots]);

  const handleRemoveFolder = useCallback(
    async (path: string) => {
      const newFolders = folders.filter((f) => f !== path);
      setFolders(newFolders);
      await invoke("remove_folder", { path });
      if (activeFolder === path) {
        // Switch to "All Folders"
        setActiveFolder(null);
        await invoke("set_active_folder", { path: null });
        if (newFolders.length > 0) {
          scanRoots(newFolders);
        } else {
          setRepos([]);
        }
      }
    },
    [folders, activeFolder, scanRoots]
  );

  const handleRefresh = useCallback(() => {
    const roots = getRoots(activeFolder, folders);
    if (roots.length > 0) {
      scanRoots(roots);
    }
  }, [activeFolder, folders, getRoots, scanRoots]);

  const handleSelectRepo = useCallback(async (repo: RepoSummary, tab: string) => {
    try {
      const detail = await invoke<RepoDetail>("get_repo_detail", {
        repoPath: repo.path,
      });
      setInitialTab(tab);
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
      setFolders(settings.folders);
      // Re-scan with current folder selection
      const roots = getRoots(activeFolder, settings.folders);
      if (roots.length > 0) {
        scanRoots(roots);
      }
    },
    [activeFolder, getRoots, scanRoots]
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
            folders={folders}
            activeFolder={activeFolder}
            onSelect={handleSelectFolder}
            onAdd={handleAddFolder}
            onRemove={handleRemoveFolder}
          />
          <button
            className="btn btn-secondary"
            onClick={handleRefresh}
            disabled={scanning || folders.length === 0}
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
            {scanState.phase !== "idle" && (
              <ScanProgressBar
                phase={scanState.phase}
                total={scanState.total}
                completed={scanState.completed}
                currentRepo={scanState.currentRepo}
              />
            )}
            {repos.length === 0 && !scanning ? (
              <div className="empty-state">
                <div className="empty-state-icon">&#128269;</div>
                <h2>No repositories found</h2>
                <p>
                  Add a folder to scan for Git repositories.
                </p>
                <button className="btn btn-primary" onClick={handleAddFolder}>
                  Add Folder
                </button>
              </div>
            ) : repos.length > 0 ? (
              <>
                <FilterBar
                  filter={filter}
                  onFilterChange={setFilter}
                  repoCount={repos.length}
                  filteredCount={filteredRepos.length}
                />
                <RepoTable
                  repos={sortedRepos}
                  sortField={sortField}
                  sortDir={sortDir}
                  onSort={handleSort}
                  onSelect={handleSelectRepo}
                />
              </>
            ) : null}
          </>
        ) : selectedRepo ? (
          <RepoDetailView detail={selectedRepo} onBack={handleBackToTable} initialTab={initialTab} />
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

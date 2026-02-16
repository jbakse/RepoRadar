# RepoRadar — Product Spec (dozens of repos, no caching)

## Summary

RepoRadar is a cross platform native desktop app that scans one or more folders for Git repositories and reports repo activity, sync state, and “backup risk” from ignored-but-important files (e.g., `.env`).

## Scope and assumptions

- Typical scale: dozens of repos.
- Safe caching: should cache results for UI performance, but very conservatively to make sure that outdated cache data is never shown. Easy for user to clear cache and rescan.
- Read-only by default; RepoRadar does not mutate repos unless you later add explicit actions.

---

## Primary features (v1)

### 1) Repo discovery

- User selects one or more **root folders**.
- Recursive scan finds repos by detecting:
  - `.git/` directory
  - `.git` file (worktrees/submodules; points to a gitdir)
- Discovery exclusions (configurable, default on):
  - `node_modules/`, `.venv/`, `dist/`, `build/`, `.next/`, `.git/` (don’t recurse inside)

### 2) Per-repo summary fields

Displayed in a sortable table:

- **Folder name**, **Repo name** and **path**
- **Created** (best-effort)
  - Prefer earliest commit date if commits exist
  - Otherwise filesystem birth time when available
  - Otherwise marked “unknown"
- **Last commit**
  - timestamp, author, short hash, subject
- **Last file edited**
  - max mtime across tracked files (optionally include untracked with a toggle)
- **Uncommitted changes**
  - counts of staged / unstaged / untracked
- **Sync status**
  - current branch upstream + ahead/behind
  - optional: branches with no upstream
- **Remotes**
  - list remotes + URLs (fetch/push)

### 3) Repo detail view

Tabs/sections:

- **Overview**: all summary fields + warnings (detached HEAD, no remote, etc.)
- **Status**: staged/unstaged/untracked lists (optionally diff preview later)
- **Branches**: local branches, upstream, ahead/behind; “no upstream” flagged
- **Remotes**: remotes and URLs
- **Ignored files**: ignored-but-present files + risk classification + matched rule

### 4) Ignored-file backup risk system (core differentiator)

Goal: find ignored files that should be backed up elsewhere.

- RepoRadar identifies files that:
  1. **exist on disk**, and
  2. **Git considers ignored** (using Git’s ignore evaluation)

- User-configurable rule lists:
  - **Always flag as important** (examples, defaults)
    - `.env`, `.env.*`, `*.key`, `*.p12`, `id_rsa*`, `*.mobileprovision`
  - **Always ignore** (examples, defaults)
    - `node_modules/**`, `dist/**`, `.DS_Store`, `.next/**`
  - Optional per-repo overrides.

- Risk levels:
  - **High**: likely secrets/credentials/keys
  - **Medium**: local DB/state that might matter (`*.sqlite`, `*.db`, `.env.local`)
  - **Low**: rebuildable artifacts/caches

---

## UX

- **Workspace picker** (saved root sets): “Work”, “Personal”, etc.
- **Main table** with filters:
  - Dirty
  - Unpushed (ahead > 0)
  - Missing remote
  - High-risk ignored files
- **Manual refresh** button

---

## Technical design (simplified, no caching)

### Architecture

- **Core library**
  - scan roots → find repos
  - per repo: run git + filesystem checks → produce structured result
  - rule engine for ignored-file risk
- **GUI**: calls core in a background worker pool, updates UI

### Implementation strategy

Use `git` subprocess calls (correct and simple) with timeouts.

Representative commands (conceptual):

- Identify repo root: `git rev-parse --show-toplevel`
- Last commit: `git log -1 --format=...`
- Worktree status + ahead/behind: `git status --porcelain=v2 -b`
- Remotes: `git remote -v`
- Tracked files list: `git ls-files -z`
- Check ignore: `git check-ignore -v --stdin` (fed candidate paths)

### “Last file edited”

Without caching, this is the potentially expensive part, but with dozens of repos it’s fine if bounded:

- Default: tracked files only (`git ls-files`) then stat mtimes.
- Provide a toggle: “include untracked files” (slower, sometimes useful).
- Provide global excludes for mtime scan (e.g., skip `dist/`, `.next/` even if tracked in unusual repos).

### Concurrency and responsiveness

- Scan runs asynchronously.
- Limit parallel repo processing (e.g., 4–8 workers).
- Each repo has a per-command timeout; if exceeded, mark partial results with an error badge.

### Error/edge handling

- No commits yet
- Detached HEAD
- No upstream configured
- Worktrees (`.git` file)
- Permission errors / unreadable files

### Technology Stack

Framework: Tauri v2
Frontend: TypeScript + React (Vite bundler)
Backend: Rust (Tauri commands for system operations)
Git Integration: Invoke git CLI via Rust std::process::Command, parse output
Styling: CSS (or Tailwind CSS for utility-first styling)

Build Targets:

- macOS: .dmg / .app bundle
- Windows: .msi installer (requires WebView2, pre-installed on Win10/11)
- Linux: .AppImage / .deb

Development Environment: Use Devbox to manage the Rust toolchain (rustup, pkg-config). Node.js 22 is available in the host shell. Run devbox shell before development to ensure Rust
tools are available.

Development Setup: Use npm create tauri-app to scaffold. Frontend dev server runs via Vite with hot reload; Rust backend recompiles on change.

CI/CD: Use GitHub Actions with Tauri's publish workflow to build platform-specific binaries on each target OS.

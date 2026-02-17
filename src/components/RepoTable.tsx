import { RepoSummary, SortField, SortDirection } from "../types";
import { RelativeDate } from "./RelativeDate";

interface Props {
  repos: RepoSummary[];
  sortField: SortField;
  sortDir: SortDirection;
  onSort: (field: SortField) => void;
  onSelect: (repo: RepoSummary, tab: string) => void;
  isCompact: boolean;
}

function SortIndicator({
  field,
  currentField,
  currentDir,
}: {
  field: SortField;
  currentField: SortField;
  currentDir: SortDirection;
}) {
  if (field !== currentField) return <span className="sort-indicator" />;
  return (
    <span className="sort-indicator active">
      {currentDir === "asc" ? "\u25B2" : "\u25BC"}
    </span>
  );
}

const HEADERS: Record<string, { wide: string; compact: string }> = {
  name: { wide: "Repository", compact: "Repo" },
  last_edited: { wide: "Last Edited", compact: "Edited" },
  last_commit: { wide: "Last Commit", compact: "Commit" },
  changes: { wide: "Changed Files", compact: "Changes" },
  sync: { wide: "Branches", compact: "Sync" },
  risk: { wide: "Ignored Files", compact: "Risk" },
};

export function RepoTable({ repos, sortField, sortDir, onSort, onSelect, isCompact }: Props) {
  const h = (key: string) => isCompact ? HEADERS[key].compact : HEADERS[key].wide;

  return (
    <div className="repo-table-wrapper">
      <table className="repo-table">
        <thead>
          <tr>
            <th onClick={() => onSort("name")}>
              {h("name")}
              <SortIndicator field="name" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("last_edited")}>
              {h("last_edited")}
              <SortIndicator field="last_edited" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("last_commit")}>
              {h("last_commit")}
              <SortIndicator field="last_commit" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("changes")}>
              {h("changes")}
              <SortIndicator field="changes" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("sync")}>
              {h("sync")}
              <SortIndicator field="sync" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("risk")}>
              {h("risk")}
              <SortIndicator field="risk" currentField={sortField} currentDir={sortDir} />
            </th>
          </tr>
        </thead>
        <tbody>
          {repos.map((repo) => (
            <RepoRow key={repo.path} repo={repo} onSelect={(tab) => onSelect(repo, tab)} isCompact={isCompact} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function RepoRow({ repo, onSelect, isCompact }: { repo: RepoSummary; onSelect: (tab: string) => void; isCompact: boolean }) {
  const totalChanges =
    repo.uncommitted.staged + repo.uncommitted.unstaged + repo.uncommitted.untracked;

  const syncLabel = getSyncLabel(repo, isCompact);

  return (
    <tr className="repo-row">
      <td className="repo-name-cell" onClick={() => onSelect("overview")}>
        <div className="repo-name">{repo.repo_name}</div>
        <div className="repo-path">{repo.path}</div>
      </td>
      <td onClick={() => onSelect("overview")}>
        {repo.last_file_edited ? (
          <>
            <RelativeDate iso={repo.last_file_edited.mtime} className="edited-date" compact={isCompact} />
            <div className="edited-file">{repo.last_file_edited.path}</div>
          </>
        ) : (
          <span className="no-data">--</span>
        )}
      </td>
      <td className="commit-cell" onClick={() => onSelect("overview")}>
        {repo.last_commit ? (
          <>
            <RelativeDate iso={repo.last_commit.timestamp} className="commit-date" compact={isCompact} />
            <div className="commit-subject">{repo.last_commit.subject}</div>
          </>
        ) : (
          <span className="no-data">No commits</span>
        )}
      </td>
      <td onClick={() => onSelect("status")}>
        {totalChanges > 0 ? (
          <div className="changes-badges">
            {repo.uncommitted.staged > 0 && (
              <span className="badge badge-staged">
                {isCompact ? repo.uncommitted.staged : `${repo.uncommitted.staged} staged`}
              </span>
            )}
            {repo.uncommitted.unstaged > 0 && (
              <span className="badge badge-unstaged">
                {isCompact ? repo.uncommitted.unstaged : `${repo.uncommitted.unstaged} modified`}
              </span>
            )}
            {repo.uncommitted.untracked > 0 && (
              <span className="badge badge-untracked">
                {isCompact ? repo.uncommitted.untracked : `${repo.uncommitted.untracked} untracked`}
              </span>
            )}
          </div>
        ) : (
          <span className="badge badge-clean">{isCompact ? "\u2713" : "Clean"}</span>
        )}
      </td>
      <td onClick={() => onSelect("branches")}>
        <div className={`sync-status ${syncLabel.className}`}>{syncLabel.text}</div>
      </td>
      <td onClick={() => onSelect("ignored")}>
        {repo.has_high_risk_ignored ? (
          <span className="badge badge-risk-high">{isCompact ? "\u2717" : "High Risk"}</span>
        ) : (
          <span className="badge badge-risk-ok">{isCompact ? "\u2713" : "OK"}</span>
        )}
      </td>
    </tr>
  );
}

function getSyncLabel(repo: RepoSummary, isCompact: boolean): { text: string; className: string } {
  const sync = repo.sync_status;

  if (sync.detached) {
    return { text: isCompact ? "\u26A0" : "Detached HEAD", className: "sync-warning" };
  }

  if (!sync.branch) {
    return { text: isCompact ? "\u26A0" : "No branch", className: "sync-warning" };
  }

  if (repo.remotes.length === 0) {
    return { text: isCompact ? "\u26A0" : "No remote", className: "sync-warning" };
  }

  if (!sync.upstream) {
    return { text: isCompact ? "\u26A0" : "No upstream", className: "sync-warning" };
  }

  if (sync.ahead > 0 && sync.behind > 0) {
    return {
      text: `${sync.ahead}\u2191 ${sync.behind}\u2193`,
      className: "sync-diverged",
    };
  }

  if (sync.ahead > 0) {
    return { text: isCompact ? `${sync.ahead}\u2191` : `${sync.ahead}\u2191 ahead`, className: "sync-ahead" };
  }

  if (sync.behind > 0) {
    return { text: isCompact ? `${sync.behind}\u2193` : `${sync.behind}\u2193 behind`, className: "sync-behind" };
  }

  if (sync.branches_without_upstream.length > 0) {
    const count = sync.branches_without_upstream.length;
    return {
      text: isCompact ? "\u26A0" : `${count} unpushed branch${count > 1 ? "es" : ""}`,
      className: "sync-warning",
    };
  }

  return { text: isCompact ? "\u2713" : "In sync", className: "sync-ok" };
}

import { RepoSummary, SortField, SortDirection } from "../types";
import { formatRelative } from "../utils";

interface Props {
  repos: RepoSummary[];
  sortField: SortField;
  sortDir: SortDirection;
  onSort: (field: SortField) => void;
  onSelect: (repo: RepoSummary) => void;
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

export function RepoTable({ repos, sortField, sortDir, onSort, onSelect }: Props) {
  return (
    <div className="repo-table-wrapper">
      <table className="repo-table">
        <thead>
          <tr>
            <th onClick={() => onSort("name")}>
              Repository
              <SortIndicator field="name" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("last_commit")}>
              Last Commit
              <SortIndicator field="last_commit" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("last_edited")}>
              Last Edited
              <SortIndicator field="last_edited" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("changes")}>
              Changes
              <SortIndicator field="changes" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("sync")}>
              Sync
              <SortIndicator field="sync" currentField={sortField} currentDir={sortDir} />
            </th>
            <th onClick={() => onSort("risk")}>
              Risk
              <SortIndicator field="risk" currentField={sortField} currentDir={sortDir} />
            </th>
          </tr>
        </thead>
        <tbody>
          {repos.map((repo) => (
            <RepoRow key={repo.path} repo={repo} onClick={() => onSelect(repo)} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function RepoRow({ repo, onClick }: { repo: RepoSummary; onClick: () => void }) {
  const totalChanges =
    repo.uncommitted.staged + repo.uncommitted.unstaged + repo.uncommitted.untracked;

  const syncLabel = getSyncLabel(repo);

  return (
    <tr className="repo-row" onClick={onClick}>
      <td className="repo-name-cell">
        <div className="repo-name">{repo.repo_name}</div>
        <div className="repo-path">{repo.path}</div>
      </td>
      <td className="commit-cell">
        {repo.last_commit ? (
          <>
            <div className="commit-subject">{repo.last_commit.subject}</div>
            <div className="commit-meta">
              <span className="commit-hash">{repo.last_commit.hash}</span>
              <span className="commit-date">
                {formatRelative(repo.last_commit.timestamp)}
              </span>
            </div>
          </>
        ) : (
          <span className="no-data">No commits</span>
        )}
      </td>
      <td>
        {repo.last_file_edited ? (
          <>
            <div className="edited-file">{repo.last_file_edited.path}</div>
            <div className="edited-date">
              {formatRelative(repo.last_file_edited.mtime)}
            </div>
          </>
        ) : (
          <span className="no-data">--</span>
        )}
      </td>
      <td>
        {totalChanges > 0 ? (
          <div className="changes-badges">
            {repo.uncommitted.staged > 0 && (
              <span className="badge badge-staged">
                {repo.uncommitted.staged} staged
              </span>
            )}
            {repo.uncommitted.unstaged > 0 && (
              <span className="badge badge-unstaged">
                {repo.uncommitted.unstaged} modified
              </span>
            )}
            {repo.uncommitted.untracked > 0 && (
              <span className="badge badge-untracked">
                {repo.uncommitted.untracked} untracked
              </span>
            )}
          </div>
        ) : (
          <span className="badge badge-clean">Clean</span>
        )}
      </td>
      <td>
        <div className={`sync-status ${syncLabel.className}`}>{syncLabel.text}</div>
      </td>
      <td>
        {repo.has_high_risk_ignored ? (
          <span className="badge badge-risk-high">High Risk</span>
        ) : (
          <span className="badge badge-risk-ok">OK</span>
        )}
      </td>
    </tr>
  );
}

function getSyncLabel(repo: RepoSummary): { text: string; className: string } {
  const sync = repo.sync_status;

  if (sync.detached) {
    return { text: "Detached HEAD", className: "sync-warning" };
  }

  if (!sync.branch) {
    return { text: "No branch", className: "sync-warning" };
  }

  if (repo.remotes.length === 0) {
    return { text: "No remote", className: "sync-warning" };
  }

  if (!sync.upstream) {
    return { text: "No upstream", className: "sync-warning" };
  }

  if (sync.ahead > 0 && sync.behind > 0) {
    return {
      text: `${sync.ahead}\u2191 ${sync.behind}\u2193`,
      className: "sync-diverged",
    };
  }

  if (sync.ahead > 0) {
    return { text: `${sync.ahead}\u2191 ahead`, className: "sync-ahead" };
  }

  if (sync.behind > 0) {
    return { text: `${sync.behind}\u2193 behind`, className: "sync-behind" };
  }

  return { text: "In sync", className: "sync-ok" };
}

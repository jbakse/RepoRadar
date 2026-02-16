import { useState } from "react";
import { RepoDetail } from "../types";
import { formatDate } from "../utils";

interface Props {
  detail: RepoDetail;
  onBack: () => void;
}

type Tab = "overview" | "status" | "branches" | "remotes" | "ignored";

export function RepoDetailView({ detail, onBack }: Props) {
  const [activeTab, setActiveTab] = useState<Tab>("overview");
  const { summary } = detail;

  const tabs: { key: Tab; label: string; badge?: number }[] = [
    { key: "overview", label: "Overview" },
    {
      key: "status",
      label: "Status",
      badge:
        summary.uncommitted.staged +
        summary.uncommitted.unstaged +
        summary.uncommitted.untracked,
    },
    { key: "branches", label: "Branches", badge: detail.branches.length },
    { key: "remotes", label: "Remotes", badge: summary.remotes.length },
    {
      key: "ignored",
      label: "Ignored Files",
      badge: detail.ignored_files.length,
    },
  ];

  return (
    <div className="repo-detail">
      <div className="detail-header">
        <button className="btn btn-back" onClick={onBack}>
          &larr; Back
        </button>
        <div className="detail-title">
          <h2>{summary.repo_name}</h2>
          <span className="detail-path">{summary.path}</span>
        </div>
      </div>

      {detail.warnings.length > 0 && (
        <div className="warnings">
          {detail.warnings.map((w, i) => (
            <div key={i} className="warning-item">
              &#9888; {w}
            </div>
          ))}
        </div>
      )}

      <div className="detail-tabs">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            className={`tab-btn ${activeTab === tab.key ? "active" : ""}`}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
            {tab.badge !== undefined && tab.badge > 0 && (
              <span className="tab-badge">{tab.badge}</span>
            )}
          </button>
        ))}
      </div>

      <div className="detail-content">
        {activeTab === "overview" && <OverviewTab detail={detail} />}
        {activeTab === "status" && <StatusTab detail={detail} />}
        {activeTab === "branches" && <BranchesTab detail={detail} />}
        {activeTab === "remotes" && <RemotesTab detail={detail} />}
        {activeTab === "ignored" && <IgnoredTab detail={detail} />}
      </div>
    </div>
  );
}

function OverviewTab({ detail }: { detail: RepoDetail }) {
  const { summary } = detail;

  return (
    <div className="overview-grid">
      <div className="overview-card">
        <h3>Repository Info</h3>
        <dl>
          <dt>Name</dt>
          <dd>{summary.repo_name}</dd>
          <dt>Folder</dt>
          <dd>{summary.folder_name}</dd>
          <dt>Path</dt>
          <dd className="monospace">{summary.path}</dd>
          <dt>Created</dt>
          <dd>{summary.created ? formatDate(summary.created) : "Unknown"}</dd>
        </dl>
      </div>

      <div className="overview-card">
        <h3>Last Commit</h3>
        {summary.last_commit ? (
          <dl>
            <dt>Hash</dt>
            <dd className="monospace">{summary.last_commit.hash}</dd>
            <dt>Subject</dt>
            <dd>{summary.last_commit.subject}</dd>
            <dt>Author</dt>
            <dd>{summary.last_commit.author}</dd>
            <dt>Date</dt>
            <dd>{formatDate(summary.last_commit.timestamp)}</dd>
          </dl>
        ) : (
          <p className="no-data">No commits yet</p>
        )}
      </div>

      <div className="overview-card">
        <h3>Sync Status</h3>
        <dl>
          <dt>Branch</dt>
          <dd>
            {summary.sync_status.detached
              ? "Detached HEAD"
              : summary.sync_status.branch || "None"}
          </dd>
          <dt>Upstream</dt>
          <dd>{summary.sync_status.upstream || "None"}</dd>
          <dt>Ahead / Behind</dt>
          <dd>
            {summary.sync_status.ahead} / {summary.sync_status.behind}
          </dd>
        </dl>
      </div>

      <div className="overview-card">
        <h3>Uncommitted Changes</h3>
        <dl>
          <dt>Staged</dt>
          <dd>{summary.uncommitted.staged}</dd>
          <dt>Modified</dt>
          <dd>{summary.uncommitted.unstaged}</dd>
          <dt>Untracked</dt>
          <dd>{summary.uncommitted.untracked}</dd>
        </dl>
      </div>
    </div>
  );
}

function StatusTab({ detail }: { detail: RepoDetail }) {
  return (
    <div className="status-lists">
      <FileList title="Staged Files" files={detail.staged_files} className="staged" />
      <FileList
        title="Modified Files (unstaged)"
        files={detail.unstaged_files}
        className="unstaged"
      />
      <FileList
        title="Untracked Files"
        files={detail.untracked_files}
        className="untracked"
      />
      {detail.staged_files.length === 0 &&
        detail.unstaged_files.length === 0 &&
        detail.untracked_files.length === 0 && (
          <p className="no-data">Working tree is clean</p>
        )}
    </div>
  );
}

function FileList({
  title,
  files,
  className,
}: {
  title: string;
  files: string[];
  className: string;
}) {
  if (files.length === 0) return null;

  return (
    <div className={`file-list file-list-${className}`}>
      <h3>
        {title} <span className="file-count">({files.length})</span>
      </h3>
      <ul>
        {files.map((f) => (
          <li key={f} className="monospace">
            {f}
          </li>
        ))}
      </ul>
    </div>
  );
}

function BranchesTab({ detail }: { detail: RepoDetail }) {
  return (
    <div className="branches-list">
      {detail.branches.length === 0 ? (
        <p className="no-data">No branches found</p>
      ) : (
        <table className="detail-table">
          <thead>
            <tr>
              <th>Branch</th>
              <th>Upstream</th>
              <th>Ahead</th>
              <th>Behind</th>
            </tr>
          </thead>
          <tbody>
            {detail.branches.map((b) => (
              <tr
                key={b.name}
                className={b.is_current ? "current-branch" : ""}
              >
                <td>
                  {b.is_current && <span className="current-indicator">*</span>}
                  {b.name}
                </td>
                <td>
                  {b.upstream || (
                    <span className="no-upstream-flag">No upstream</span>
                  )}
                </td>
                <td>{b.ahead > 0 ? b.ahead : "--"}</td>
                <td>{b.behind > 0 ? b.behind : "--"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function RemotesTab({ detail }: { detail: RepoDetail }) {
  const { remotes } = detail.summary;

  return (
    <div className="remotes-list">
      {remotes.length === 0 ? (
        <p className="no-data">No remotes configured</p>
      ) : (
        <table className="detail-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Fetch URL</th>
              <th>Push URL</th>
            </tr>
          </thead>
          <tbody>
            {remotes.map((r) => (
              <tr key={r.name}>
                <td className="remote-name">{r.name}</td>
                <td className="monospace">{r.fetch_url}</td>
                <td className="monospace">{r.push_url}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function IgnoredTab({ detail }: { detail: RepoDetail }) {
  return (
    <div className="ignored-list">
      {detail.ignored_files.length === 0 ? (
        <p className="no-data">No risky ignored files found</p>
      ) : (
        <table className="detail-table">
          <thead>
            <tr>
              <th>File</th>
              <th>Risk</th>
              <th>Category</th>
              <th>Matched Rule</th>
            </tr>
          </thead>
          <tbody>
            {detail.ignored_files.map((f) => (
              <tr key={f.path} className={`risk-${f.risk_level.toLowerCase()}`}>
                <td className="monospace">{f.path}</td>
                <td>
                  <span
                    className={`badge badge-risk-${f.risk_level.toLowerCase()}`}
                  >
                    {f.risk_level}
                  </span>
                </td>
                <td>{f.category}</td>
                <td className="monospace">{f.matched_rule}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

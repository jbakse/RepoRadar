import { useState } from "react";
import { RepoDetail } from "../types";
import { formatDate } from "../utils";
import { RelativeDate } from "./RelativeDate";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

interface Props {
  detail: RepoDetail;
  onBack: () => void;
  initialTab?: string;
}

type Tab = "overview" | "status" | "branches" | "remotes" | "ignored";

const validTabs: Tab[] = ["overview", "status", "branches", "remotes", "ignored"];

export function RepoDetailView({ detail, onBack, initialTab }: Props) {
  const startTab = validTabs.includes(initialTab as Tab) ? (initialTab as Tab) : "overview";
  const [activeTab, setActiveTab] = useState<Tab>(startTab);
  const { summary } = detail;

  const tabs: { key: Tab; label: string; badge?: number }[] = [
    { key: "overview", label: "Overview" },
    {
      key: "status",
      label: "Changed Files",
      badge: detail.changed_files.length,
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

  const handleOpenPath = () => {
    revealItemInDir(summary.path).catch(console.error);
  };

  return (
    <div className="overview-section">
      <div className="overview-card">
        <h3>Repository Info</h3>
        <dl>
          <dt>Name</dt>
          <dd>{summary.repo_name}</dd>
          <dt>Folder</dt>
          <dd>{summary.folder_name}</dd>
          <dt>Path</dt>
          <dd>
            <a className="path-link" onClick={handleOpenPath}>
              {summary.path}
            </a>
          </dd>
          <dt>Created</dt>
          <dd>{summary.created ? formatDate(summary.created) : "Unknown"}</dd>
        </dl>
      </div>

      <div className="overview-card">
        <h3>Recent Commits</h3>
        {detail.recent_commits.length > 0 ? (
          <table className="detail-table">
            <thead>
              <tr>
                <th>Author</th>
                <th>Date</th>
                <th>Message</th>
              </tr>
            </thead>
            <tbody>
              {detail.recent_commits.map((c, i) => (
                <tr key={i}>
                  <td>{c.author}</td>
                  <td>
                    <RelativeDate iso={c.timestamp} />
                  </td>
                  <td className="commit-message">{c.subject}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="no-data">No commits yet</p>
        )}
      </div>
    </div>
  );
}

function StatusTab({ detail }: { detail: RepoDetail }) {
  if (detail.changed_files.length === 0) {
    return <p className="no-data">Working tree is clean</p>;
  }

  return (
    <div className="changed-files-list">
      <table className="detail-table">
        <thead>
          <tr>
            <th>File Name</th>
            <th>Status</th>
            <th>Path</th>
          </tr>
        </thead>
        <tbody>
          {detail.changed_files.map((f, i) => (
            <tr key={`${f.path}-${f.staged}-${i}`}>
              <td className="monospace">{f.file_name}</td>
              <td>
                <span className={`badge badge-status-${f.status.toLowerCase()}`}>
                  {f.status}
                </span>
                {f.staged && (
                  <span className="badge badge-staged-indicator">Staged</span>
                )}
              </td>
              <td className="monospace file-path-cell">{f.path}</td>
            </tr>
          ))}
        </tbody>
      </table>
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
        <p className="no-data">No ignored files found</p>
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
              <tr
                key={f.path}
                className={f.risk_level !== "Low" ? `risk-${f.risk_level.toLowerCase()}` : ""}
              >
                <td className="monospace">
                  {f.is_directory ? `${f.path}/` : f.path}
                  {f.is_directory && f.child_count > 0 && (
                    <span className="child-count">({f.child_count} files)</span>
                  )}
                </td>
                <td>
                  {f.risk_level !== "Low" ? (
                    <span className={`badge badge-risk-${f.risk_level.toLowerCase()}`}>
                      {f.risk_level}
                    </span>
                  ) : (
                    <span className="text-muted">--</span>
                  )}
                </td>
                <td>{f.risk_level !== "Low" ? f.category : ""}</td>
                <td className="monospace">{f.matched_rule || ""}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

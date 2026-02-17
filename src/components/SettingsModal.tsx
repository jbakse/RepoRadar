import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../types";

interface Props {
  onClose: () => void;
  onSave: (settings: AppSettings) => void;
}

export function SettingsModal({ onClose, onSave }: Props) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [activeTab, setActiveTab] = useState<
    "general" | "flag" | "ignore" | "exclusions"
  >("general");

  // Editable text areas (one pattern per line)
  const [flagPatterns, setFlagPatterns] = useState("");
  const [ignorePatterns, setIgnorePatterns] = useState("");
  const [exclusions, setExclusions] = useState("");

  useEffect(() => {
    async function load() {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
      setFlagPatterns(s.always_flag_patterns.join("\n"));
      setIgnorePatterns(s.always_ignore_patterns.join("\n"));
      setExclusions(s.discovery_exclusions.join("\n"));
    }
    load();
  }, []);

  if (!settings) return null;

  const handleSave = () => {
    const updated: AppSettings = {
      ...settings,
      always_flag_patterns: flagPatterns
        .split("\n")
        .map((s) => s.trim())
        .filter(Boolean),
      always_ignore_patterns: ignorePatterns
        .split("\n")
        .map((s) => s.trim())
        .filter(Boolean),
      discovery_exclusions: exclusions
        .split("\n")
        .map((s) => s.trim())
        .filter(Boolean),
    };
    onSave(updated);
    onClose();
  };

  const handleReset = () => {
    // Reset to default values
    const defaults: Partial<AppSettings> = {
      always_flag_patterns: [
        ".env",
        ".env.*",
        "*.key",
        "*.pem",
        "*.p12",
        "id_rsa*",
        "*.mobileprovision",
      ],
      always_ignore_patterns: [
        "node_modules/**",
        "dist/**",
        ".DS_Store",
        ".next/**",
        "__pycache__/**",
        "*.pyc",
        "target/**",
      ],
      discovery_exclusions: ["node_modules", ".venv", "dist", "build", ".next"],
    };

    if (activeTab === "flag") {
      setFlagPatterns(defaults.always_flag_patterns!.join("\n"));
    } else if (activeTab === "ignore") {
      setIgnorePatterns(defaults.always_ignore_patterns!.join("\n"));
    } else if (activeTab === "exclusions") {
      setExclusions(defaults.discovery_exclusions!.join("\n"));
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Settings</h2>
          <button className="modal-close" onClick={onClose}>
            &times;
          </button>
        </div>

        <div className="settings-tabs">
          <button
            className={`settings-tab ${activeTab === "general" ? "active" : ""}`}
            onClick={() => setActiveTab("general")}
          >
            General
          </button>
          <button
            className={`settings-tab ${activeTab === "flag" ? "active" : ""}`}
            onClick={() => setActiveTab("flag")}
          >
            High-Risk Patterns
          </button>
          <button
            className={`settings-tab ${activeTab === "ignore" ? "active" : ""}`}
            onClick={() => setActiveTab("ignore")}
          >
            Ignore Patterns
          </button>
          <button
            className={`settings-tab ${activeTab === "exclusions" ? "active" : ""}`}
            onClick={() => setActiveTab("exclusions")}
          >
            Scan Exclusions
          </button>
        </div>

        <div className="modal-body">
          {activeTab === "general" && (
            <div className="settings-section">
              <label className="settings-toggle">
                <input
                  type="checkbox"
                  checked={settings.include_untracked_mtime}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      include_untracked_mtime: e.target.checked,
                    })
                  }
                />
                <span>Include untracked files in "last edited" time</span>
              </label>
              <p className="settings-help">
                When enabled, untracked files (not yet added to git) will be
                considered when calculating the most recently edited file in each
                repository.
              </p>
            </div>
          )}

          {activeTab === "flag" && (
            <div className="settings-section">
              <p className="settings-help">
                Glob patterns for files that should be flagged as high-risk when
                found in .gitignore. These are typically secrets, keys, and
                credentials. One pattern per line.
              </p>
              <textarea
                className="settings-textarea"
                value={flagPatterns}
                onChange={(e) => setFlagPatterns(e.target.value)}
                rows={10}
                placeholder=".env&#10;*.key&#10;*.pem"
              />
            </div>
          )}

          {activeTab === "ignore" && (
            <div className="settings-section">
              <p className="settings-help">
                Glob patterns for gitignored files that are safe to ignore (build
                artifacts, caches, etc). Files matching these patterns won't
                appear in risk reports. One pattern per line.
              </p>
              <textarea
                className="settings-textarea"
                value={ignorePatterns}
                onChange={(e) => setIgnorePatterns(e.target.value)}
                rows={10}
                placeholder="node_modules/**&#10;dist/**&#10;.DS_Store"
              />
            </div>
          )}

          {activeTab === "exclusions" && (
            <div className="settings-section">
              <p className="settings-help">
                Directory names to skip when scanning for repositories. These are
                matched by name, not glob pattern. One directory name per line.
              </p>
              <textarea
                className="settings-textarea"
                value={exclusions}
                onChange={(e) => setExclusions(e.target.value)}
                rows={10}
                placeholder="node_modules&#10;.venv&#10;dist"
              />
            </div>
          )}
        </div>

        <div className="modal-footer">
          {activeTab !== "general" && (
            <button className="btn btn-secondary" onClick={handleReset}>
              Reset to Defaults
            </button>
          )}
          <div className="modal-footer-right">
            <button className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button className="btn btn-primary" onClick={handleSave}>
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

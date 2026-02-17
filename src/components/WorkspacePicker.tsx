import { useState, useRef, useEffect } from "react";

interface Props {
  folders: string[];
  activeFolder: string | null; // null = "All Folders"
  onSelect: (folder: string | null) => void;
  onAdd: () => void;
  onRemove: (folder: string) => void;
}

function folderBasename(path: string): string {
  const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export function WorkspacePicker({
  folders,
  activeFolder,
  onSelect,
  onAdd,
  onRemove,
}: Props) {
  const [showDropdown, setShowDropdown] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!showDropdown) return;
    function handleClick(e: MouseEvent) {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        setShowDropdown(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [showDropdown]);

  const label = activeFolder
    ? folderBasename(activeFolder)
    : folders.length > 0
      ? "All Folders"
      : "Select Folder";

  return (
    <div className="workspace-picker" ref={pickerRef}>
      <button
        className="btn btn-workspace"
        onClick={() => setShowDropdown(!showDropdown)}
      >
        {label}
        <span className="dropdown-arrow">
          {showDropdown ? "\u25B2" : "\u25BC"}
        </span>
      </button>

      {showDropdown && (
        <div className="workspace-dropdown">
          {folders.length > 0 && (
            <div
              className={`workspace-item ${activeFolder === null ? "active" : ""}`}
            >
              <button
                className="workspace-item-btn"
                onClick={() => {
                  onSelect(null);
                  setShowDropdown(false);
                }}
              >
                <span className="workspace-name">All Folders</span>
              </button>
            </div>
          )}

          {folders.map((folder) => (
            <div
              key={folder}
              className={`workspace-item ${activeFolder === folder ? "active" : ""}`}
            >
              <button
                className="workspace-item-btn"
                onClick={() => {
                  onSelect(folder);
                  setShowDropdown(false);
                }}
              >
                <span className="workspace-name">
                  {folderBasename(folder)}
                </span>
                <span className="folder-path">{folder}</span>
              </button>
              <button
                className="workspace-delete"
                onClick={(e) => {
                  e.stopPropagation();
                  onRemove(folder);
                }}
                title="Remove folder"
              >
                &#128465;
              </button>
            </div>
          ))}

          {folders.length > 0 && <div className="workspace-divider" />}

          <button
            className="workspace-item-btn workspace-new"
            onClick={() => {
              onAdd();
              setShowDropdown(false);
            }}
          >
            + Add Folder
          </button>
        </div>
      )}
    </div>
  );
}

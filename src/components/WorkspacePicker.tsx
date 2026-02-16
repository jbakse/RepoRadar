import { useState } from "react";
import { Workspace } from "../types";

interface Props {
  workspaces: Workspace[];
  activeWorkspace: Workspace | null;
  onSelect: (workspace: Workspace) => void;
  onCreate: (name: string) => void;
  onDelete: (name: string) => void;
}

export function WorkspacePicker({
  workspaces,
  activeWorkspace,
  onSelect,
  onCreate,
  onDelete,
}: Props) {
  const [showDropdown, setShowDropdown] = useState(false);
  const [newName, setNewName] = useState("");
  const [creating, setCreating] = useState(false);

  const handleCreate = () => {
    if (newName.trim()) {
      onCreate(newName.trim());
      setNewName("");
      setCreating(false);
      setShowDropdown(false);
    }
  };

  return (
    <div className="workspace-picker">
      <button
        className="btn btn-workspace"
        onClick={() => setShowDropdown(!showDropdown)}
      >
        {activeWorkspace ? activeWorkspace.name : "Select Workspace"}
        <span className="dropdown-arrow">{showDropdown ? "\u25B2" : "\u25BC"}</span>
      </button>

      {showDropdown && (
        <div className="workspace-dropdown">
          {workspaces.map((ws) => (
            <div
              key={ws.name}
              className={`workspace-item ${
                activeWorkspace?.name === ws.name ? "active" : ""
              }`}
            >
              <button
                className="workspace-item-btn"
                onClick={() => {
                  onSelect(ws);
                  setShowDropdown(false);
                }}
              >
                <span className="workspace-name">{ws.name}</span>
                <span className="workspace-roots">
                  {ws.roots.length} folder{ws.roots.length !== 1 ? "s" : ""}
                </span>
              </button>
              <button
                className="workspace-delete"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(ws.name);
                }}
                title="Delete workspace"
              >
                x
              </button>
            </div>
          ))}

          {workspaces.length > 0 && <div className="workspace-divider" />}

          {creating ? (
            <div className="workspace-create-form">
              <input
                type="text"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                placeholder="Workspace name..."
                autoFocus
              />
              <button className="btn btn-small" onClick={handleCreate}>
                Create
              </button>
              <button
                className="btn btn-small"
                onClick={() => {
                  setCreating(false);
                  setNewName("");
                }}
              >
                Cancel
              </button>
            </div>
          ) : (
            <button
              className="workspace-item-btn workspace-new"
              onClick={() => setCreating(true)}
            >
              + New Workspace
            </button>
          )}
        </div>
      )}
    </div>
  );
}

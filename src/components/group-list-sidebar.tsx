import { useState } from "react";
import { Plus, X, Pencil } from "lucide-react";
import type { Partner } from "../types";
import CreateGroupDialog from "./create-group-dialog";
import { createPartner, deletePartner, renamePartner } from "../lib/tauri-api";

interface Props {
  groups: Partner[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onRefresh: () => void;
}

export default function GroupListSidebar({ groups, selectedId, onSelect, onRefresh }: Props) {
  const [showCreate, setShowCreate] = useState(false);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async (name: string) => {
    try {
      await createPartner(name);
      onRefresh();
      setShowCreate(false);
      setError(null);
    } catch (e) {
      console.error("Failed to create partner:", e);
      setError(String(e));
    }
  };

  const startRename = (g: Partner) => {
    setRenamingId(g.id);
    setRenameValue(g.name);
  };

  const submitRename = async (id: string) => {
    if (renameValue.trim()) {
      try {
        await renamePartner(id, renameValue.trim());
        onRefresh();
        setError(null);
      } catch (e) {
        console.error("Failed to rename partner:", e);
        setError(String(e));
      }
    }
    setRenamingId(null);
  };

  const handleDelete = async (id: string, name: string) => {
    if (!window.confirm(`Delete partner "${name}" and all its members?`)) return;
    try {
      await deletePartner(id);
      onRefresh();
      setError(null);
    } catch (e) {
      console.error("Failed to delete partner:", e);
      setError(String(e));
    }
  };

  return (
    <div
      style={{
        width: 200,
        minWidth: 200,
        background: "var(--color-bg-sidebar-light)",
        borderRight: "1px solid var(--color-border-light)",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "12px 12px 8px",
          borderBottom: "1px solid var(--color-border-light)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <span
          className="section-header"
          style={{ marginBottom: 0, color: "var(--color-text-on-light)" }}
        >
          Partner List
        </span>
        <button
          className="btn-icon"
          onClick={() => setShowCreate(true)}
          title="Create partner"
          style={{ width: 24, height: 24, color: "var(--color-text-on-light)" }}
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Error message */}
      {error && (
        <div style={{ padding: "6px 12px", fontSize: "var(--font-size-xs)", color: "var(--color-accent-danger)", background: "var(--color-bg-table-row-alt)" }}>
          {error}
        </div>
      )}

      {/* Partner list */}
      <div style={{ flex: 1, overflowY: "auto" }}>
        {groups.length === 0 ? (
          <div style={{ padding: 16, color: "var(--color-text-muted-light)", fontSize: "var(--font-size-sm)" }}>
            No partners yet
          </div>
        ) : (
          groups.map((g) => (
            <div
              key={g.id}
              style={{
                padding: "8px 12px",
                cursor: "pointer",
                background: selectedId === g.id ? "var(--color-bg-table-hover)" : "transparent",
                borderBottom: "1px solid var(--color-border-light)",
                borderLeft: selectedId === g.id ? "3px solid var(--color-accent-primary)" : "3px solid transparent",
                display: "flex",
                alignItems: "center",
                gap: 6,
              }}
              onClick={() => onSelect(g.id)}
              onMouseEnter={(e) => {
                if (selectedId !== g.id) {
                  (e.currentTarget as HTMLDivElement).style.background = "var(--color-bg-table-hover)";
                }
              }}
              onMouseLeave={(e) => {
                if (selectedId !== g.id) {
                  (e.currentTarget as HTMLDivElement).style.background = "transparent";
                }
              }}
            >
              {renamingId === g.id ? (
                <input
                  autoFocus
                  value={renameValue}
                  onChange={(e) => setRenameValue(e.target.value)}
                  onBlur={() => submitRename(g.id)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") submitRename(g.id);
                    if (e.key === "Escape") setRenamingId(null);
                  }}
                  onClick={(e) => e.stopPropagation()}
                  style={{ flex: 1, padding: "2px 4px", height: 24 }}
                />
              ) : (
                <>
                  <span style={{ flex: 1, fontSize: "var(--font-size-base)", color: "var(--color-text-on-light)" }}>
                    {g.name}
                  </span>
                  <span className="badge badge-default">{g.member_count ?? 0}</span>
                  <button
                    className="btn-icon"
                    title="Rename"
                    onClick={(e) => { e.stopPropagation(); startRename(g); }}
                    style={{ width: 20, height: 20, color: "var(--color-text-muted-light)" }}
                  >
                    <Pencil size={11} />
                  </button>
                  <button
                    className="btn-icon"
                    title="Delete"
                    onClick={(e) => { e.stopPropagation(); handleDelete(g.id, g.name); }}
                    style={{ width: 20, height: 20, color: "var(--color-accent-danger)" }}
                  >
                    <X size={11} />
                  </button>
                </>
              )}
            </div>
          ))
        )}
      </div>

      {showCreate && (
        <CreateGroupDialog
          onConfirm={handleCreate}
          onCancel={() => setShowCreate(false)}
        />
      )}
    </div>
  );
}

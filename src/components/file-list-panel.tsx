import { FilePlus, X } from "lucide-react";
import { selectFiles } from "../lib/tauri-api";
import type { FileStatus } from "../types";

interface Props {
  files: string[];
  onFilesChange: (files: string[]) => void;
  label?: string;
  fileStatuses?: Record<string, FileStatus>;
}

const STATUS_COLORS: Record<FileStatus, string> = {
  pending: "#e65100",
  encrypting: "#0097b8",
  decrypting: "#0097b8",
  done: "#16a34a",
  warning: "#b45309",
  error: "#dc2626",
};

function FileStatusBadge({ filePath, status }: { filePath: string; status?: FileStatus }) {
  if (!status) return null;
  const color = STATUS_COLORS[status] ?? "#64748b";
  const label = status === "pending" ? "—"
    : status === "encrypting" ? "enc…"
    : status === "decrypting" ? "dec…"
    : status === "done" ? "✓"
    : status === "warning" ? "warn"
    : "err";
  return (
    <span style={{ fontSize: "var(--font-size-xs)", color, fontWeight: 600 }}
      title={`${filePath}: ${status}`}>
      {label}
    </span>
  );
}

export default function FileListPanel({
  files, onFilesChange, label = "Source Files", fileStatuses,
}: Props) {
  const addFiles = async () => {
    const result = await selectFiles();
    if (result) onFilesChange([...new Set([...files, ...result])]);
  };

  const removeFile = (path: string) => onFilesChange(files.filter((f) => f !== path));
  const getFileName = (path: string) => path.replace(/\\/g, "/").split("/").pop() ?? path;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span className="section-title" style={{ marginBottom: 0, color: "var(--color-text-muted-light)" }}>
          {label} ({files.length})
        </span>
        <div style={{ display: "flex", gap: 6 }}>
          <button className="btn btn-primary" onClick={addFiles} style={{ height: 28, padding: "0 12px", fontSize: "var(--font-size-sm)" }}>
            <FilePlus size={14} /> Add
          </button>
          {files.length > 0 && (
            <button className="btn btn-ghost" onClick={() => onFilesChange([])}
              style={{ height: 28, padding: "0 10px", fontSize: "var(--font-size-sm)" }}>
              Clear
            </button>
          )}
        </div>
      </div>

      <div className="table-container" style={{ minHeight: 80, maxHeight: 220, overflowY: "auto" }}>
        <table>
          <thead>
            <tr>
              <th>File Name</th>
              <th>File Path</th>
              {fileStatuses !== undefined && <th style={{ width: 40 }}>Status</th>}
              <th style={{ width: 40 }}></th>
            </tr>
          </thead>
          <tbody>
            {files.length === 0 ? (
              <tr>
                <td
                  colSpan={fileStatuses !== undefined ? 4 : 3}
                  style={{
                    padding: "28px 12px",
                    textAlign: "center",
                    color: "var(--color-text-muted-light)",
                    fontSize: "var(--font-size-sm)",
                  }}
                >
                  Click &quot;Add&quot; to select files
                </td>
              </tr>
            ) : (
              files.map((f) => (
                <tr key={f}>
                  <td style={{ color: "var(--color-text-on-light)" }}>{getFileName(f)}</td>
                  <td style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-muted-light)", maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {f}
                  </td>
                  {fileStatuses && (
                    <td style={{ width: 40, textAlign: "center" }}>
                      <FileStatusBadge filePath={f} status={fileStatuses[f]} />
                    </td>
                  )}
                  <td style={{ width: 40, textAlign: "right" }}>
                    <button className="btn-icon" onClick={() => removeFile(f)} title="Remove file"
                      style={{ color: "var(--color-text-muted-light)", width: 24, height: 24 }}>
                      <X size={12} />
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

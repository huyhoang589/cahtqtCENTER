import type { EncryptProgress, EncryptResult } from "../types";

interface Props {
  progress: EncryptProgress[];
  result: EncryptResult | null;
  isRunning: boolean;
}

export default function EncryptProgressPanel({ progress, result, isRunning }: Props) {
  const last = progress[progress.length - 1];
  const pct = last && last.total > 0 ? Math.round((last.current / last.total) * 100) : 0;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <span className="section-title" style={{ color: "var(--color-text-muted-light)" }}>Progress</span>

      {/* Progress bar */}
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      {last && (
        <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-muted-light)" }}>
          {last.current} / {last.total} ({pct}%)
          {last.status === "processing" && (
            <span style={{ marginLeft: 8 }}>{last.file_name}</span>
          )}
        </div>
      )}

      {/* Live progress log with sticky summary header (A-1) */}
      <div
        style={{
          background: "var(--color-bg-log)",
          border: "1px solid var(--color-border-dark)",
          borderRadius: "var(--radius-sm)",
          height: 200,
          overflowY: "auto",
          padding: "0",
          position: "relative",
        }}
      >
        {/* Sticky summary header — shown when result exists */}
        {result && (
          <div style={{
            position: "sticky", top: 0, zIndex: 1,
            background: "var(--color-bg-log)",
            padding: "4px 10px",
            borderBottom: "1px solid var(--color-border-dark)",
            display: "flex", gap: 12,
          }}>
            <span style={{ color: "var(--color-accent-success)", fontSize: "var(--font-size-xs)" }}>✓ {result.success_count} success</span>
            {result.error_count > 0 && (
              <span style={{ color: "var(--color-accent-danger)", fontSize: "var(--font-size-xs)" }}>✗ {result.error_count} failed</span>
            )}
          </div>
        )}

        {progress.length === 0 ? (
          <div style={{ padding: 16, color: "var(--color-text-secondary)", textAlign: "center", fontSize: "var(--font-size-sm)" }}>
            No operations yet
          </div>
        ) : (
          [...progress].reverse().slice(0, 50).map((p) => (
            <div
              key={`${p.file_name}-${p.current}-${p.status}`}
              style={{
                padding: "3px 10px",
                fontSize: "var(--font-size-xs)",
                fontFamily: "var(--font-mono)",
                color:
                  p.status === "success" ? "var(--color-text-log-success)" :
                  p.status === "warning" ? "var(--color-text-log-warning)" :
                  p.status === "error"   ? "var(--color-text-log-error)" :
                  "var(--color-text-log-info)",
                display: "flex",
                gap: 8,
              }}
            >
              <span>
                {p.status === "processing" ? "⋯" :
                 p.status === "success" ? "✓" :
                 p.status === "warning" ? "⚠" : "✗"}
              </span>
              <span style={{ flex: 1 }}>{p.file_name}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

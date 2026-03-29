import { CheckCircle, XCircle } from "lucide-react";
import type { DecryptProgress, DecryptResult } from "../types";

interface Props {
  progress: DecryptProgress[];
  result: DecryptResult | null;
}

export default function DecryptProgressPanel({ progress, result }: Props) {
  const last = progress[progress.length - 1];
  const pct = last && last.total > 0 ? Math.round((last.current / last.total) * 100) : 0;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <span className="section-title" style={{ color: "var(--color-text-muted-light)" }}>Progress</span>

      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      {last && (
        <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-muted-light)" }}>
          {last.current} / {last.total} ({pct}%)
        </div>
      )}

      {result && (
        <div
          style={{
            background: "var(--color-bg-surface)",
            borderRadius: "var(--radius-sm)",
            padding: "10px 12px",
            // B-3: colored border — green if all success, red if any error
            border: `1px solid ${result.error_count > 0 ? "#fca5a5" : "#86efac"}`,
          }}
        >
          <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
            <span style={{ display: "flex", alignItems: "center", gap: 4, color: "var(--color-accent-success)", fontSize: "var(--font-size-sm)" }}>
              <CheckCircle size={14} /> {result.success_count} success
            </span>
            {result.error_count > 0 && (
              <span style={{ display: "flex", alignItems: "center", gap: 4, color: "var(--color-accent-danger)", fontSize: "var(--font-size-sm)" }}>
                <XCircle size={14} /> {result.error_count} failed
              </span>
            )}
          </div>
          {result.errors.slice(0, 5).map((e, i) => (
            <div key={i} style={{ color: "var(--color-accent-danger)", fontSize: "var(--font-size-xs)", marginTop: 4 }}>{e}</div>
          ))}
        </div>
      )}

    </div>
  );
}

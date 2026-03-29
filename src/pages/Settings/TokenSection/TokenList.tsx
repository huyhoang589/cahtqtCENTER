import type { MechanismDetail, TokenInfo } from "../../../types";
import MechanismTable from "./MechanismTable";

interface Props {
  tokens: TokenInfo[];
  mechanisms: MechanismDetail[];
}

export default function TokenList({ tokens, mechanisms }: Props) {
  if (tokens.length === 0) return null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      {tokens.map((token) => (
        <div
          key={token.slot_id}
          style={{
            border: "1px solid var(--color-border-light)",
            borderRadius: "var(--radius-sm)",
            padding: "10px 14px",
            background: "var(--color-bg-table-row)",
          }}
        >
          {/* PIN locked — critical error banner */}
          {token.user_pin_locked && (
            <div
              style={{
                color: "var(--color-accent-danger)",
                fontSize: "var(--font-size-sm)",
                marginBottom: 8,
              }}
            >
              ✗ Token PIN is locked. Contact administrator to unlock using PUK.
            </div>
          )}

          {/* PIN final try — warning banner */}
          {!token.user_pin_locked && token.user_pin_final_try && (
            <div
              style={{
                color: "var(--color-accent-warning)",
                fontSize: "var(--font-size-sm)",
                marginBottom: 8,
              }}
            >
              ⚠ Only 1 PIN attempt remaining. Do NOT enter PIN incorrectly.
            </div>
          )}

          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-on-light)" }}>
            <strong>Slot {token.slot_id}</strong> — {token.label || "(no label)"}
          </div>
          <div
            style={{
              fontSize: "var(--font-size-xs)",
              color: "var(--color-text-muted-light)",
              marginTop: 2,
            }}
          >
            Serial: {token.serial_number} · Model: {token.model}
          </div>
        </div>
      ))}
      {/* Mechanism table renders once after all token cards — mechanisms are per-scan, not per-token */}
      <MechanismTable mechanisms={mechanisms} />
    </div>
  );
}

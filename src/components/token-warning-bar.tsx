import { useTokenStatusContext } from "../contexts/token-status-context";

interface Props {
  onLogin: () => void;
}

/// Orange warning bar shown when token is not logged in.
/// Returns null when status === "logged_in" (no warning needed).
/// Shared by EncryptPage and DecryptPage.
export default function TokenWarningBar({ onLogin }: Props) {
  const { status } = useTokenStatusContext();

  if (status === "logged_in") return null;

  return (
    <div
      style={{
        background: "var(--color-bg-dialog-token)",
        borderBottom: "1px solid var(--color-border-dialog-token)",
        padding: "8px 20px",
        display: "flex",
        alignItems: "center",
        gap: 12,
        flexShrink: 0,
        color: "#ffffff",
      }}
    >
      <span style={{ fontSize: "var(--font-size-sm)", flex: 1 }}>
        Token not logged in — login required to encrypt or decrypt
      </span>
      <button
        onClick={onLogin}
        style={{
          height: 28,
          padding: "0 12px",
          background: "rgba(0,0,0,0.3)",
          color: "#ffffff",
          border: "1px solid rgba(255,255,255,0.3)",
          borderRadius: "var(--radius-sm)",
          fontSize: "var(--font-size-sm)",
          cursor: "pointer",
          whiteSpace: "nowrap",
        }}
      >
        Login Now
      </button>
    </div>
  );
}

import { Shield } from "lucide-react";
import { useTokenStatusContext } from "../contexts/token-status-context";

export default function AppHeader() {
  const { dll_found, status, cert_cn } = useTokenStatusContext();

  // 4-state dot: no dll → red, disconnected → gray, connected → orange, logged_in → green
  const dotClass = !dll_found
    ? "status-dot status-dot-disconnected"
    : status === "logged_in"
    ? "status-dot status-dot-connected"
    : status === "connected"
    ? "status-dot status-dot-warning"
    : "status-dot status-dot-idle";

  const label = !dll_found
    ? "htqt lib not found"
    : status === "logged_in"
    ? "Token logged in"
    : status === "connected"
    ? "Token connected"
    : "Token not found";

  return (
    <header
      style={{
        height: 56,
        flexShrink: 0,
        background: "var(--color-bg-window)",
        borderBottom: "1px solid var(--color-border-dark)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0 20px",
      }}
    >
      {/* Left: logo + app name */}
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <Shield size={22} color="var(--color-accent-primary)" />
        <span
          style={{
            fontSize: "var(--font-size-2xl)",
            fontWeight: "var(--font-weight-bold)",
            color: "var(--color-text-primary)",
          }}
        >
          CAHTQT PKI
        </span>
      </div>

      {/* Right: token status */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          fontSize: "var(--font-size-sm)",
          color: "var(--color-text-secondary)",
        }}
      >
        <span className={dotClass} />
        <span>{label}</span>
        {status === "logged_in" && cert_cn && (
          <span className="cert-cn-badge">{cert_cn}</span>
        )}
      </div>
    </header>
  );
}

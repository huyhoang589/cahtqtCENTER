import { useNavigate } from "react-router-dom";
import TokenWarningBar from "../components/token-warning-bar";
import { useTokenStatusContext } from "../contexts/token-status-context";
import { useLicenseGen } from "../hooks/use-license-gen";

export default function LicenseGenPage() {
  const navigate = useNavigate();
  const { status: tokenStatus } = useTokenStatusContext();
  const {
    credential, expiresAt, setExpiresAt,
    isPerpetual, setIsPerpetual,
    unitName, setUnitName,
    isGenerating, result, auditEntries,
    handleImport, handleGenerate,
    handleExport, handleDelete, handleOpenFolder,
  } = useLicenseGen();

  const canGenerate =
    tokenStatus === "logged_in" && credential?.isValid && !isGenerating;

  // Convert Unix timestamp ↔ date input string
  const expiryDateStr = expiresAt
    ? new Date(expiresAt * 1000).toISOString().slice(0, 10)
    : "";
  const handleExpiryChange = (val: string) => {
    const ts = Math.floor(new Date(val).getTime() / 1000);
    if (!isNaN(ts)) setExpiresAt(ts);
  };

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <TokenWarningBar onLogin={() => navigate("/settings")} />

      {/* Header */}
      <div style={{ padding: "16px 20px 12px", borderBottom: "1px solid var(--color-border-light)", display: "flex", justifyContent: "space-between", alignItems: "center", flexShrink: 0 }}>
        <h2 style={{ fontSize: "var(--font-size-xl)", color: "var(--color-text-on-light)" }}>License Gen</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-ghost" onClick={handleImport}>
            Import Credential…
          </button>
          <button className="primary" onClick={handleGenerate} disabled={!canGenerate}>
            {isGenerating ? "Generating…" : "Generate License"}
          </button>
        </div>
      </div>

      {/* Body */}
      <div style={{ flex: 1, padding: 20, overflowY: "auto", display: "flex", flexDirection: "column", gap: 16 }}>
        {/* Status message */}
        {result && (
          <div style={{ padding: "8px 12px", borderRadius: "var(--radius-md)", background: result.success ? "var(--color-success-bg, #e6f9e6)" : "var(--color-error-bg, #fde8e8)", color: result.success ? "var(--color-success, #16a34a)" : "var(--color-error, #dc2626)", fontSize: "var(--font-size-sm)" }}>
            {result.success ? `License saved to ${result.outputPath}` : result.error}
          </div>
        )}

        {/* Preview cards: Credential (left) + License Payload (right) */}
        {credential && (
          <div style={{ display: "flex", gap: 16 }}>
            {/* Left: Credential Preview */}
            <div style={{ flex: 1, border: "1px solid var(--color-border-light)", borderRadius: "var(--radius-md)", padding: 16 }}>
              <div style={{ fontWeight: "bold", marginBottom: 8, color: "#000" }}>Credential Preview</div>
              {!credential.isValid && (
                <div style={{ color: "var(--color-error, #dc2626)", fontSize: "var(--font-size-xs)", marginBottom: 8 }}>
                  {credential.validationError}
                </div>
              )}
              <div style={{ display: "grid", gridTemplateColumns: "120px 1fr", gap: "4px 12px", fontSize: "var(--font-size-sm)" }}>
                <span style={{ color: "#333" }}>User</span>
                <span style={{ color: "#000" }}>{credential.credential.userName}</span>
                <span style={{ color: "#333" }}>Token Serial</span>
                <span style={{ color: "#000", fontFamily: "monospace", fontSize: "var(--font-size-xs)" }}>{credential.credential.tokenSerial}</span>
                <span style={{ color: "#333" }}>CPU ID</span>
                <span style={{ color: "#000", fontFamily: "monospace", fontSize: "var(--font-size-xs)" }}>{credential.credential.cpuId}</span>
                <span style={{ color: "#333" }}>Board Serial</span>
                <span style={{ color: "#000", fontFamily: "monospace", fontSize: "var(--font-size-xs)" }}>{credential.credential.boardSerial}</span>
                <span style={{ color: "#333" }}>Registered At</span>
                <span style={{ color: "#000" }}>{credential.credential.registeredAt}</span>
              </div>
            </div>

            {/* Right: License Payload Preview */}
            <div style={{ flex: 1, border: "1px solid var(--color-border-light)", borderRadius: "var(--radius-md)", padding: 16 }}>
              <div style={{ fontWeight: "bold", marginBottom: 8, color: "#000" }}>License Preview</div>
              {result?.success ? (
                <div style={{ display: "grid", gridTemplateColumns: "120px 1fr", gap: "4px 12px", fontSize: "var(--font-size-sm)" }}>
                  <span style={{ color: "#333" }}>Machine FP</span>
                  <span style={{ color: "#000", fontFamily: "monospace", fontSize: "var(--font-size-xs)" }}>{result.machineFp || credential.machineFp}</span>
                  <span style={{ color: "#333" }}>Output Path</span>
                  <span style={{ color: "#000", fontSize: "var(--font-size-xs)", wordBreak: "break-all" }}>{result.outputPath}</span>
                </div>
              ) : (
                <div style={{ color: "#999", fontSize: "var(--font-size-sm)" }}>Generate a license to preview</div>
              )}
            </div>
          </div>
        )}

        {/* Generation controls */}
        <div style={{ display: "flex", gap: 16, alignItems: "center", flexWrap: "wrap" }}>
          <label style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
            Unit Name
            <input type="text" value={unitName} onChange={(e) => setUnitName(e.target.value)} style={{ marginLeft: 8, padding: "4px 8px", borderRadius: "var(--radius-sm)", border: "1px solid var(--color-border-light)", background: "var(--color-bg-input, #fff)", color: "var(--color-text-on-light)", fontSize: "var(--font-size-sm)", width: 160 }} />
          </label>
          <label style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
            Expires
            <input type="date" value={expiryDateStr} onChange={(e) => handleExpiryChange(e.target.value)} disabled={isPerpetual} style={{ marginLeft: 8, padding: "4px 8px", borderRadius: "var(--radius-sm)", border: "1px solid var(--color-border-light)", background: "var(--color-bg-input, #fff)", color: "var(--color-text-on-light)", fontSize: "var(--font-size-sm)" }} />
          </label>
          <label style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)", display: "flex", alignItems: "center", gap: 4 }}>
            <input type="checkbox" checked={isPerpetual} onChange={(e) => setIsPerpetual(e.target.checked)} />
            Perpetual
          </label>
        </div>

        {/* Audit history table */}
        <div>
          <h3 style={{ fontSize: "var(--font-size-base)", color: "var(--color-text-on-light)", marginBottom: 8 }}>License History</h3>
          <div style={{ border: "1px solid var(--color-border-light)", borderRadius: "var(--radius-md)", overflow: "hidden" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
              <thead>
                <tr style={{ background: "var(--color-bg-sidebar)", borderBottom: "1px solid var(--color-border-light)" }}>
                  {["Date", "User", "Unit", "Token", "Machine FP", "Expiry", "Actions"].map((h) => (
                    <th key={h} style={{ padding: "8px 10px", textAlign: "left", fontWeight: "var(--font-weight-medium)", color: "var(--color-text-secondary)" }}>{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {auditEntries.length === 0 && (
                  <tr><td colSpan={7} style={{ padding: "16px 10px", textAlign: "center", color: "var(--color-text-secondary)" }}>No licenses generated yet</td></tr>
                )}
                {auditEntries.map((e) => (
                  <tr key={e.id} style={{ borderBottom: "1px solid var(--color-border-light)" }}>
                    <td style={{ padding: "6px 10px" }}>{new Date(e.createdAt * 1000).toLocaleDateString()}</td>
                    <td style={{ padding: "6px 10px" }}>{e.userName}</td>
                    <td style={{ padding: "6px 10px" }}>{e.unitName}</td>
                    <td style={{ padding: "6px 10px", fontFamily: "monospace" }}>{e.tokenSerial}</td>
                    <td style={{ padding: "6px 10px", fontFamily: "monospace", fontSize: "var(--font-size-xs)" }}>{e.machineFp}</td>
                    <td style={{ padding: "6px 10px" }}>{e.expiresAt ? new Date(e.expiresAt * 1000).toLocaleDateString() : "Perpetual"}</td>
                    <td style={{ padding: "6px 10px" }}>
                      <div style={{ display: "flex", gap: 4 }}>
                        <button className="btn btn-ghost" style={{ fontSize: "var(--font-size-xs)", padding: "2px 6px" }} onClick={() => handleExport(e.id)} disabled={!e.licenseBlob} title={!e.licenseBlob ? "No license data stored" : "Export to file"}>Export</button>
                        <button className="btn btn-ghost" style={{ fontSize: "var(--font-size-xs)", padding: "2px 6px" }} onClick={() => handleOpenFolder(e.userName)} title="Open LICENSE folder">Open Folder</button>
                        <button className="btn btn-ghost" style={{ fontSize: "var(--font-size-xs)", padding: "2px 6px", color: "var(--color-error, #dc2626)" }} onClick={() => { if (window.confirm(`Delete license for ${e.userName}?`)) handleDelete(e.id); }}>Delete</button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}

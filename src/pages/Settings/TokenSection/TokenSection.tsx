import { useState } from "react";
import { logoutToken, tokenSetLibraryPath } from "../../../lib/tauri-api";
import { useTokenStatusContext } from "../../../contexts/token-status-context";
import LoginTokenModal from "../../../components/login-token-modal";
import { useTokenScan } from "./useTokenScan";
import CertificateTable from "./CertificateTable";
import LibraryPathInput from "./LibraryPathInput";
import LibraryStatus from "./LibraryStatus";
import ScanButton from "./ScanButton";
import TokenList from "./TokenList";

interface Props {
  pkcs11Mode: "auto" | "manual";
  pkcs11ManualPath: string;
  onPkcs11ModeChange: (mode: "auto" | "manual") => Promise<void>;
  onPkcs11ManualPathChange: (path: string) => Promise<void>;
}

export default function TokenSection({ pkcs11Mode, pkcs11ManualPath, onPkcs11ModeChange, onPkcs11ManualPathChange }: Props) {
  const {
    scanStatus,
    scanResult,
    selectedCert,
    senderCert,
    libraryInfo,
    error,
    scan,
    selectCert,
    exportAsSender,
    clearSender,
  } = useTokenScan(pkcs11Mode, pkcs11ManualPath);

  const { status: tokenStatus, cert_cn, refresh } = useTokenStatusContext();
  const [loginModalOpen, setLoginModalOpen] = useState(false);

  // Wrap scan to also refresh token status immediately after
  const handleScan = async () => {
    await scan();
    refresh();
  };

  const senderCertSerial = senderCert?.serial ?? null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <LibraryPathInput
        mode={pkcs11Mode}
        manualPath={pkcs11ManualPath}
        onModeChange={onPkcs11ModeChange}
        onManualPathChange={(path) => onPkcs11ManualPathChange(path).catch(() => {})}
        onManualPathBrowse={async () => {
          if (pkcs11ManualPath) {
            try { await tokenSetLibraryPath(pkcs11ManualPath); } catch { /* ignore */ }
          }
        }}
        onManualPathClear={() => onPkcs11ManualPathChange("").catch(() => {})}
      />
      <div style={{ borderBottom: "1px solid var(--color-border-light)", margin: "8px 0" }} />
      <div className="section-title" style={{ color: "var(--color-text-muted-light)" }}>
        eToken / Hardware Token
      </div>

      {/* PKCS#11 library status */}
      <LibraryStatus libraryInfo={libraryInfo} />

      {/* Scan + Login Token actions */}
      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <ScanButton status={scanStatus} onScan={handleScan} />
          <button
            onClick={() => setLoginModalOpen(true)}
            disabled={tokenStatus !== "connected"}
            style={{ opacity: tokenStatus === "connected" ? 1 : 0.4 }}
          >
            Login Token
          </button>
          {tokenStatus === "logged_in" && (
            <button
              className="btn btn-ghost"
              onClick={async () => { await logoutToken(); refresh(); }}
            >
              Logout
            </button>
          )}
        </div>

        {scanStatus.type === "done" && !scanStatus.found && (
          <div
            style={{
              color: "var(--color-text-muted-light)",
              fontSize: "var(--font-size-sm)",
            }}
          >
            No token detected. Connect your eToken and scan again.
          </div>
        )}
        {error && (
          <div className="text-error">{error}</div>
        )}
      </div>

      {/* Token authenticated status row */}
      {tokenStatus === "logged_in" && cert_cn && (
        <div
          style={{
            padding: "10px 14px",
            background: "rgba(139,195,74,0.1)",
            border: "1px solid var(--color-accent-success)",
            borderRadius: "var(--radius-sm)",
            fontSize: "var(--font-size-sm)",
            display: "flex",
            alignItems: "center",
            gap: 8,
            color: "var(--color-accent-success)",
          }}
        >
          <span>✓ Token authenticated — ready for Encrypt / Decrypt</span>
          <span className="cert-cn-badge">{cert_cn}</span>
        </div>
      )}

      {/* Detected token cards + mechanism support table */}
      {scanResult && scanResult.tokens.length > 0 && (
        <TokenList tokens={scanResult.tokens} mechanisms={scanResult.mechanisms ?? []} />
      )}

      {/* Certificate table */}
      {scanResult && scanResult.certificates.length > 0 && (
        <CertificateTable
          entries={scanResult.certificates}
          selectedCertId={selectedCert?.certObjectId ?? null}
          senderCertSerial={senderCertSerial}
          onSelect={selectCert}
          onExport={exportAsSender}
        />
      )}

      {/* Current sender certificate status */}
      {senderCert && (
        <div
          style={{
            marginTop: 8,
            padding: "12px 14px",
            background: "var(--color-bg-table-row)",
            border: "1px solid var(--color-accent-success)",
            borderRadius: "var(--radius-sm)",
            fontSize: "var(--font-size-sm)",
          }}
        >
          <div
            style={{
              fontWeight: "var(--font-weight-semibold)",
              color: "var(--color-accent-success)",
              marginBottom: 6,
            }}
          >
            ✓ Current Sender Certificate
          </div>
          <div style={{ color: "var(--color-text-on-light)" }}>{senderCert.display_name}</div>
          <div style={{ color: "var(--color-text-muted-light)" }}>{senderCert.email}</div>
          <div style={{ color: "var(--color-text-muted-light)" }}>
            Valid until: {senderCert.valid_until}
          </div>
          <button className="btn btn-ghost" onClick={clearSender} style={{ marginTop: 8 }}>
            × Clear
          </button>
        </div>
      )}

      <LoginTokenModal
        open={loginModalOpen}
        onClose={() => setLoginModalOpen(false)}
        onSuccess={() => refresh()}
      />
    </div>
  );
}

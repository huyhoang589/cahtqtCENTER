import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import type { CertInfo } from "../types";
import { addPartnerMember, importCertPreview, selectCertFile } from "../lib/tauri-api";

interface Props {
  partnerId: string;
  onAdded: () => void;
  onCancel: () => void;
}

export default function AddRecipientDialog({ partnerId, onAdded, onCancel }: Props) {
  const [certPath, setCertPath] = useState<string | null>(null);
  const [certInfo, setCertInfo] = useState<CertInfo | null>(null);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const selectCert = async () => {
    const files = await selectCertFile();
    if (!files || files.length === 0) return;
    const path = files[0];
    setLoading(true);
    setError(null);
    try {
      const info = await importCertPreview(path);
      setCertPath(path);
      setCertInfo(info);
      setName(info.cn);
      setEmail(info.email ?? "");
    } catch (e) {
      setError(`Failed to parse certificate: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = async () => {
    if (!certPath || !certInfo) return;
    setLoading(true);
    setError(null);
    try {
      await addPartnerMember(partnerId, certPath, name || undefined, email || undefined);
      onAdded();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (ts: number) => new Date(ts * 1000).toLocaleDateString();
  const isExpired = certInfo ? certInfo.valid_to < Math.floor(Date.now() / 1000) : false;

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open) onCancel(); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content
          className="dialog-content"
          style={{ width: 460 }}
          aria-describedby={undefined}
        >
          <div className="dialog-header">
            <Dialog.Title className="dialog-title">Add Partner Member</Dialog.Title>
          </div>

          <div className="dialog-body" style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {/* Certificate picker */}
            <button
              className="btn btn-ghost"
              onClick={selectCert}
              disabled={loading}
              style={{ width: "100%" }}
            >
              {certPath ? "Change Certificate" : "Select Certificate (.crt / .pem / .cer / .der)"}
            </button>

            {/* Cert preview */}
            {certInfo && (
              <div
                style={{
                  background: "var(--color-bg-window)",
                  borderRadius: "var(--radius-sm)",
                  padding: "10px 12px",
                  border: `1px solid ${isExpired ? "var(--color-accent-danger)" : "var(--color-border-dark)"}`,
                  fontSize: "var(--font-size-sm)",
                }}
              >
                <div style={{ marginBottom: 4, fontWeight: "var(--font-weight-semibold)" }}>
                  Certificate Preview
                </div>
                {isExpired && (
                  <div style={{ color: "var(--color-accent-danger)", marginBottom: 6 }}>
                    ⚠ This certificate is expired
                  </div>
                )}
                <div>CN: {certInfo.cn}</div>
                <div>Serial: {certInfo.serial}</div>
                <div>Valid: {formatDate(certInfo.valid_from)} → {formatDate(certInfo.valid_to)}</div>
                {certInfo.email && <div>Email: {certInfo.email}</div>}
              </div>
            )}

            {/* Name / email fields */}
            {certInfo && (
              <>
                <input
                  placeholder="Display name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
                <input
                  placeholder="Email (optional)"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
              </>
            )}

            {error && <div className="text-error">{error}</div>}
          </div>

          <div className="dialog-footer">
            <button className="btn btn-ghost" onClick={onCancel}>Cancel</button>
            <button
              className="btn btn-primary"
              onClick={handleAdd}
              disabled={!certPath || !certInfo || loading}
            >
              {loading ? "Adding…" : "Add"}
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

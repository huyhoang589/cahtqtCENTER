import { useEffect, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Eye, EyeOff } from "lucide-react";
import { loginToken } from "../lib/tauri-api";

interface Props {
  open: boolean;
  onClose: () => void;
  onSuccess: (certCn: string) => void;
}

/// Orange-themed Login Token dialog — shared by Settings, EncryptPage, DecryptPage.
/// Calls loginToken(pin) directly, manages its own loading/error state.
/// PIN is cleared from component state immediately after the call.
export default function LoginTokenModal({ open, onClose, onSuccess }: Props) {
  const [pin, setPin] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setPin("");
      setError(null);
      setLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const handleSubmit = async () => {
    if (!pin.trim() || loading) return;
    setLoading(true);
    setError(null);
    try {
      const result = await loginToken(pin);
      setPin(""); // clear PIN from component state immediately
      onSuccess(result.cert_cn);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSubmit();
    if (e.key === "Escape") { setPin(""); onClose(); }
  };

  if (!open) return null;

  return (
    <Dialog.Root open={open} onOpenChange={(o) => { if (!o) { setPin(""); onClose(); } }}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content
          className="dialog-content dialog-content-token"
          style={{ width: 380 }}
          aria-describedby={undefined}
        >
          <div style={{ padding: "32px", textAlign: "center" }}>
            <div style={{ fontSize: 48, marginBottom: 12 }}>🔑</div>
            <Dialog.Title
              style={{
                fontSize: "var(--font-size-lg)",
                fontWeight: "var(--font-weight-bold)",
                color: "var(--color-text-primary)",
                marginBottom: 4,
              }}
            >
              Login Token
            </Dialog.Title>
            <p style={{ fontSize: "var(--font-size-md)", color: "var(--color-token-dialog-hint)", marginBottom: 24 }}>
              Enter your token PIN to authenticate
            </p>

            <label
              style={{
                display: "block",
                fontSize: "var(--font-size-sm)",
                color: "var(--color-text-primary)",
                textAlign: "left",
                marginBottom: 6,
                fontWeight: "var(--font-weight-medium)",
              }}
            >
              Token PIN
            </label>
            <div style={{ position: "relative", marginBottom: 8 }}>
              <input
                ref={inputRef}
                type={showPin ? "text" : "password"}
                value={pin}
                onChange={(e) => setPin(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Enter PIN"
                autoComplete="off"
                disabled={loading}
                style={{ marginBottom: 0, height: 36, borderRadius: "var(--radius-sm)", paddingRight: 36 }}
              />
              <button
                type="button"
                onClick={() => setShowPin((v) => !v)}
                style={{
                  position: "absolute",
                  right: 8,
                  top: "50%",
                  transform: "translateY(-50%)",
                  background: "transparent",
                  border: "none",
                  color: "var(--color-text-secondary)",
                  padding: 0,
                  cursor: "pointer",
                  width: 20,
                  height: 20,
                  display: "flex",
                  alignItems: "center",
                }}
              >
                {showPin ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>

            {error && (
              <div
                style={{
                  color: "var(--color-token-dialog-error)",
                  fontSize: "var(--font-size-sm)",
                  marginBottom: 12,
                  textAlign: "left",
                  background: "rgba(0,0,0,0.2)",
                  padding: "6px 10px",
                  borderRadius: "var(--radius-sm)",
                }}
              >
                {error}
              </div>
            )}

            <button
              type="button"
              onClick={handleSubmit}
              disabled={!pin.trim() || loading}
              style={{
                width: "100%",
                height: 44,
                background: "var(--color-token-dialog-btn)",
                color: "var(--color-text-primary)",
                border: "none",
                borderRadius: "var(--radius-sm)",
                fontSize: "var(--font-size-md)",
                fontWeight: "var(--font-weight-semibold)",
                cursor: !pin.trim() || loading ? "not-allowed" : "pointer",
                opacity: !pin.trim() || loading ? 0.5 : 1,
                marginBottom: 8,
              }}
            >
              {loading ? "Authenticating…" : "Login"}
            </button>
            <button
              type="button"
              onClick={() => { setPin(""); onClose(); }}
              disabled={loading}
              style={{
                width: "100%",
                height: 44,
                background: "var(--color-token-dialog-btn)",
                color: "var(--color-text-primary)",
                border: "none",
                borderRadius: "var(--radius-sm)",
                fontSize: "var(--font-size-md)",
                fontWeight: "var(--font-weight-semibold)",
                cursor: "pointer",
              }}
            >
              Cancel
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

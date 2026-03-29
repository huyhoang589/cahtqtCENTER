import { useEffect, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";

interface Props {
  onConfirm: (pin: string) => void;
  onCancel: () => void;
  /** "standard" = cyan theme (default), "token" = orange government theme */
  variant?: "standard" | "token";
}

export default function PinDialog({ onConfirm, onCancel, variant = "standard" }: Props) {
  const [pin, setPin] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const isToken = variant === "token";

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!pin) return;
    const captured = pin;
    setPin("");
    onConfirm(captured);
  };

  // Focus input when dialog opens
  useEffect(() => {
    const t = setTimeout(() => inputRef.current?.focus(), 50);
    return () => clearTimeout(t);
  }, []);

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open) { setPin(""); onCancel(); } }}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content
          className={isToken ? "dialog-content dialog-content-token" : "dialog-content"}
          style={isToken ? { width: 380 } : { width: 340 }}
          aria-describedby={undefined}
        >
          {isToken ? (
            /* ── Token / orange variant ── */
            <div style={{ padding: "32px", textAlign: "center" }}>
              {/* Key icon */}
              <div style={{ fontSize: 48, marginBottom: 12 }}>🔑</div>
              <Dialog.Title
                style={{
                  fontSize: "var(--font-size-lg)",
                  fontWeight: "var(--font-weight-bold)",
                  color: "var(--color-text-primary)",
                  marginBottom: 4,
                }}
              >
                Token Login
              </Dialog.Title>
              <p style={{ fontSize: "var(--font-size-md)", color: "var(--color-token-dialog-hint)", marginBottom: 24 }}>
                Enter your token password
              </p>
              <form onSubmit={handleSubmit}>
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
                  Token Password
                </label>
                <input
                  ref={inputRef}
                  type="password"
                  placeholder="Enter password"
                  value={pin}
                  onChange={(e) => setPin(e.target.value)}
                  autoComplete="off"
                  style={{ marginBottom: 12, height: 36, borderRadius: "var(--radius-sm)" }}
                />
                <button
                  type="submit"
                  disabled={!pin}
                  style={{
                    width: "100%",
                    height: 44,
                    background: "var(--color-token-dialog-btn)",
                    color: "var(--color-text-primary)",
                    border: "none",
                    borderRadius: "var(--radius-sm)",
                    fontSize: "var(--font-size-md)",
                    fontWeight: "var(--font-weight-semibold)",
                    cursor: pin ? "pointer" : "not-allowed",
                    opacity: pin ? 1 : 0.5,
                    marginBottom: 8,
                  }}
                >
                  Login
                </button>
                <button
                  type="button"
                  onClick={() => { setPin(""); onCancel(); }}
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
                  Exit
                </button>
              </form>
            </div>
          ) : (
            /* ── Standard / cyan variant ── */
            <>
              <div className="dialog-header">
                <Dialog.Title className="dialog-title">Enter Token PIN</Dialog.Title>
              </div>
              <div className="dialog-body">
                <form onSubmit={handleSubmit}>
                  <input
                    ref={inputRef}
                    type="password"
                    placeholder="PIN"
                    value={pin}
                    onChange={(e) => setPin(e.target.value)}
                    autoComplete="off"
                    style={{ marginBottom: 4 }}
                  />
                </form>
              </div>
              <div className="dialog-footer">
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={() => { setPin(""); onCancel(); }}
                >
                  Cancel
                </button>
                <button
                  type="button"
                  className="btn btn-primary"
                  disabled={!pin}
                  onClick={() => { if (!pin) return; const captured = pin; setPin(""); onConfirm(captured); }}
                >
                  Confirm
                </button>
              </div>
            </>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

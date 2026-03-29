import { X } from "lucide-react";
import TokenSection from "./Settings/TokenSection/TokenSection";
import { useSettingsContext } from "../contexts/settings-context";
import { selectOutputDir } from "../lib/tauri-api";

export default function SettingsPage() {
  const { outputDataDir, pkcs11Mode, pkcs11ManualPath, saveOutputDataDir, savePkcs11Mode, savePkcs11ManualPath } = useSettingsContext();
  const hasOutputDir = outputDataDir.trim().length > 0;

  const handleBrowseOutputDir = async () => {
    const dir = await selectOutputDir();
    if (dir) await saveOutputDataDir(dir);
  };

  const handleClearOutputDir = async () => {
    await saveOutputDataDir("");
  };

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      {/* Header */}
      <div style={{ marginBottom: 24, paddingBottom: 16, borderBottom: "1px solid var(--color-border-light)" }}>
        <h2 style={{ fontSize: "var(--font-size-xl)", color: "var(--color-text-on-light)" }}>Settings</h2>
      </div>

      {/* OUTPUT DATA DIR section */}
      <div style={{ display: "flex", flexDirection: "column", gap: 6, marginBottom: 24 }}>
        <div className="section-title" style={{ color: "var(--color-text-muted-light)" }}>
          OUTPUT DATA DIR
        </div>
        <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-muted-light)" }}>
          Directory where encrypted/decrypted files are saved. Leave empty to use Desktop.
        </div>
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input
            type="text"
            value={outputDataDir}
            onChange={(e) => saveOutputDataDir(e.target.value).catch(() => {})}
            onBlur={(e) => saveOutputDataDir(e.target.value.trim()).catch(() => {})}
            placeholder="Default: Desktop folder"
            style={{
              flex: 1,
              fontFamily: "var(--font-mono)",
              fontSize: "var(--font-size-sm)",
              border: `1px solid ${hasOutputDir ? "var(--color-accent-primary)" : "var(--color-border-light)"}`,
              borderRadius: "var(--radius-sm)",
              padding: "6px 10px",
              background: "var(--color-bg-input, #fff)",
              color: "var(--color-text-on-light)",
            }}
          />
          {hasOutputDir && (
            <button className="btn btn-ghost" onClick={handleClearOutputDir} title="Clear output dir">
              <X size={14} />
            </button>
          )}
          <button className="btn btn-secondary" onClick={handleBrowseOutputDir}>Browse…</button>
        </div>
        <div style={{ fontSize: "var(--font-size-xs)", color: hasOutputDir ? "var(--color-accent-success)" : "var(--color-text-muted-light)" }}>
          {hasOutputDir ? `✓ Output directory: ${outputDataDir}` : "Default: Desktop folder"}
        </div>
      </div>

      <div style={{ borderBottom: "1px solid var(--color-border-light)", marginBottom: 24 }} />

      {/* eToken / PKCS#11 hardware token section */}
      <TokenSection
        pkcs11Mode={pkcs11Mode}
        pkcs11ManualPath={pkcs11ManualPath}
        onPkcs11ModeChange={savePkcs11Mode}
        onPkcs11ManualPathChange={savePkcs11ManualPath}
      />
    </div>
  );
}

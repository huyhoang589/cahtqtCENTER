import { useNavigate } from "react-router-dom";
import { FolderOpen } from "lucide-react";
import ConfirmEncryptDialog from "../components/confirm-encrypt-dialog";
import EncryptProgressPanel from "../components/encrypt-progress-panel";
import FileListPanel from "../components/file-list-panel";
import PartnerSelectPanel from "../components/partner-select-panel";
import TokenWarningBar from "../components/token-warning-bar";
import { useEncrypt } from "../hooks/use-encrypt";
import { useTokenStatusContext } from "../contexts/token-status-context";
import { useSettingsContext } from "../contexts/settings-context";
import { useFileStatuses } from "../hooks/use-file-statuses";
import { useEncryptPanelResize } from "../hooks/use-encrypt-panel-resize";
import { getAppSettings, openFolder } from "../lib/tauri-api";
import { useState } from "react";

type Step = "idle" | "confirm" | "running" | "done";

export default function EncryptPage() {
  const {
    selectedFiles, setSelectedFiles,
    selectedRecipientIds, setSelectedRecipientIds,
    isEncrypting, progress, result,
    startEncrypt, reset,
  } = useEncrypt();

  const navigate = useNavigate();
  const { dll_found, status: tokenStatus } = useTokenStatusContext();
  const { outputDataDir } = useSettingsContext();
  const { fileStatuses, resetStatuses } = useFileStatuses("encrypt-progress", "encrypt");
  const { displayRatio, containerRef, onDividerMouseDown } = useEncryptPanelResize();

  const [selectedCertPaths, setSelectedCertPaths] = useState<string[]>([]);
  const [selectedPartnerName, setSelectedPartnerName] = useState<string>("");
  const [step, setStep] = useState<Step>("idle");

  // A-3: Auto-reset when Encrypt clicked after a completed batch.
  // If step is "done", prompt user about possible duplicate output files before resetting.
  const handleEncryptClick = () => {
    if (selectedFiles.length === 0 || selectedRecipientIds.length === 0) return;
    if (step === "done") {
      const ok = window.confirm(
        "Output files from the previous batch may already exist. Start a new batch? (Existing files will be overwritten.)"
      );
      if (!ok) return;
      handleReset();
    }
    setStep("confirm");
  };

  const handleConfirm = async () => {
    setStep("running");
    resetStatuses(selectedFiles);
    const outputDir = outputDataDir
      ? `${outputDataDir}/SF/ENCRYPT/${selectedPartnerName}`
      : null;
    await startEncrypt(selectedCertPaths, selectedPartnerName, outputDir);
    setStep("done");
  };

  const handleReset = () => { reset(); setStep("idle"); };

  const handleSelectionChange = (ids: string[], certPaths: string[], partnerName: string) => {
    setSelectedRecipientIds(ids);
    setSelectedCertPaths(certPaths);
    setSelectedPartnerName(partnerName);
  };

  // A-2: Use getAppSettings() for correct output path (includes Desktop fallback from Rust)
  const handleOpenFolder = async () => {
    try {
      const settings = await getAppSettings();
      await openFolder(`${settings.output_data_dir}/SF/ENCRYPT`);
    } catch {}
  };

  const canEncrypt =
    dll_found && tokenStatus === "logged_in" &&
    selectedFiles.length > 0 && selectedRecipientIds.length > 0 && !isEncrypting;

  const showProgress = step !== "idle" || result !== null;

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <TokenWarningBar onLogin={() => navigate("/settings")} />

      {/* Header */}
      <div style={{ padding: "16px 20px 12px", borderBottom: "1px solid var(--color-border-light)", display: "flex", justifyContent: "space-between", alignItems: "center", flexShrink: 0 }}>
        <h2 style={{ fontSize: "var(--font-size-xl)", color: "var(--color-text-on-light)" }}>Encrypt Files</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-ghost" onClick={handleOpenFolder} title="Open output folder" style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <FolderOpen size={14} /> Open Folder
          </button>
          {/* A-3: "New Batch" button removed — re-clicking Encrypt auto-resets */}
          <button className="primary" onClick={handleEncryptClick} disabled={!canEncrypt}>
            {isEncrypting ? "Encrypting…" : "Encrypt"}
          </button>
        </div>
      </div>

      {/* Resizable split container */}
      <div ref={containerRef} style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* LEFT: file list + progress */}
        <div className="encrypt-split-left" style={{ width: `${displayRatio * 100}%`, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          <div style={{ flex: 1, padding: 16, overflowY: "auto" }}>
            <FileListPanel
              files={selectedFiles}
              onFilesChange={(newFiles) => { setSelectedFiles(newFiles); resetStatuses(newFiles); }}
              label="Source Files"
              fileStatuses={fileStatuses}
            />
          </div>
          {showProgress && (
            <div style={{ padding: "0 16px 16px", flexShrink: 0 }}>
              <EncryptProgressPanel progress={progress} result={result} isRunning={isEncrypting} />
            </div>
          )}
        </div>

        {/* Divider */}
        <div className="encrypt-split-divider" onMouseDown={onDividerMouseDown} />

        {/* RIGHT: partner selection */}
        <div className="encrypt-split-right" style={{ width: `${(1 - displayRatio) * 100}%`, padding: 16, overflowY: "auto", display: "flex", flexDirection: "column" }}>
          <PartnerSelectPanel selectedIds={selectedRecipientIds} onSelectionChange={handleSelectionChange} />
        </div>
      </div>

      {step === "confirm" && (
        <ConfirmEncryptDialog
          fileCount={selectedFiles.length}
          recipientCount={selectedRecipientIds.length}
          onConfirm={handleConfirm}
          onCancel={() => setStep("idle")}
        />
      )}
    </div>
  );
}

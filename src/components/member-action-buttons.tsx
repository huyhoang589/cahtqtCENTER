import { useState } from "react";
import type { PartnerMember } from "../types";
import { exportMemberCert, setCommunication } from "../lib/tauri-api";

// C-1: unified status for inline action indicator
type ActionStatus = "idle" | "loading" | "done" | "error";

interface Props {
  member: PartnerMember;
  partnerName: string;
  outputDataDir: string;
  desktopPath: string;
  onRemove: () => void;
}

export default function MemberActionButtons({ member, partnerName, outputDataDir, desktopPath, onRemove }: Props) {
  const [actionStatus, setActionStatus] = useState<ActionStatus>("idle");

  const baseDir = outputDataDir || desktopPath;

  const handleExport = async () => {
    setActionStatus("loading");
    const destDir = `${baseDir}/SF/CERT_EXPORT/${partnerName}`;
    try {
      await exportMemberCert(member.cert_file_path, destDir, member.cert_cn, member.cert_serial);
      setActionStatus("done");
    } catch {
      setActionStatus("error");
    }
    setTimeout(() => setActionStatus("idle"), 2500);
  };

  const handleSetComm = async (pin: string) => {
    setActionStatus("loading");
    const destDir = `${baseDir}/SF/SET_COMMUNICATION/${partnerName}`;
    try {
      await setCommunication(member.cert_file_path, partnerName, destDir, pin);
      setActionStatus("done");
    } catch {
      setActionStatus("error");
    }
    setTimeout(() => setActionStatus("idle"), 2500);
  };

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
      {/* Status indicator — only visible while action is in-flight */}
      {actionStatus !== "idle" && (
        <span style={{ fontSize: 11, minWidth: 14, color:
          actionStatus === "done" ? "var(--color-accent-success)" :
          actionStatus === "error" ? "var(--color-accent-danger)" :
          "var(--color-text-muted)" }}>
          {actionStatus === "loading" ? "…" : actionStatus === "done" ? "✓" : "✗"}
        </span>
      )}

      {/* 🔗 Set Communication */}
      <button
        className="btn-icon"
        onClick={() => handleSetComm("")}
        disabled={actionStatus === "loading"}
        title="Set Communication"
        style={{ color: "#6366f1" }}
        onMouseEnter={e => (e.currentTarget.style.background = "rgba(0,0,0,0.06)")}
        onMouseLeave={e => (e.currentTarget.style.background = "")}
      >🔗</button>

      {/* 📤 Export Certificate */}
      <button
        className="btn-icon"
        onClick={handleExport}
        disabled={actionStatus === "loading"}
        title="Export Certificate"
        style={{ color: "#0097b8" }}
        onMouseEnter={e => (e.currentTarget.style.background = "rgba(0,0,0,0.06)")}
        onMouseLeave={e => (e.currentTarget.style.background = "")}
      >📤</button>

      {/* × Remove Member */}
      <button
        className="btn-icon"
        onClick={onRemove}
        disabled={actionStatus === "loading"}
        title="Remove member"
        style={{ color: "var(--color-accent-danger)" }}
        onMouseEnter={e => (e.currentTarget.style.background = "rgba(0,0,0,0.06)")}
        onMouseLeave={e => (e.currentTarget.style.background = "")}
      >×</button>


    </div>
  );
}

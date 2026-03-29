import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { FileStatus, EncryptProgress, DecryptProgress } from "../types";

type ProgressPayload = EncryptProgress | DecryptProgress;
type EventName = "encrypt-progress" | "decrypt-progress";

function mapStatus(status: string, operationType: "encrypt" | "decrypt"): FileStatus {
  if (status === "processing") return operationType === "encrypt" ? "encrypting" : "decrypting";
  if (status === "success") return "done";
  if (status === "warning") return "warning";
  if (status === "error") return "error";
  return "pending";
}

export function useFileStatuses(eventName: EventName, operationType: "encrypt" | "decrypt") {
  const [fileStatuses, setFileStatuses] = useState<Record<string, FileStatus>>({});

  useEffect(() => {
    const unlistenPromise = listen<ProgressPayload>(eventName, (event) => {
      const payload = event.payload as ProgressPayload & { file_path?: string };
      if (!payload.file_path) return; // skip DLL raw progress events without file_path
      const mapped = mapStatus(payload.status, operationType);
      setFileStatuses((prev) => ({ ...prev, [payload.file_path!]: mapped }));
    });
    return () => { unlistenPromise.then((fn) => fn()); };
  }, [eventName, operationType]);

  const resetStatuses = (files: string[]) => {
    const initial: Record<string, FileStatus> = {};
    files.forEach((f) => { initial[f] = "pending"; });
    setFileStatuses(initial);
  };

  return { fileStatuses, resetStatuses };
}

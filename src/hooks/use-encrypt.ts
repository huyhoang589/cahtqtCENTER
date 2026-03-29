import { useCallback, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { EncryptProgress, EncryptResult } from "../types";
import { encryptBatch } from "../lib/tauri-api";

export function useEncrypt() {
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [selectedRecipientIds, setSelectedRecipientIds] = useState<string[]>([]);
  const [isEncrypting, setIsEncrypting] = useState(false);
  const [progress, setProgress] = useState<EncryptProgress[]>([]);
  const [result, setResult] = useState<EncryptResult | null>(null);

  /// certPaths = cert_file_paths of all selected members; partnerName = Partner.name (output folder)
  const startEncrypt = useCallback(async (certPaths: string[], partnerName: string, outputDir?: string | null) => {
    if (selectedFiles.length === 0 || certPaths.length === 0 || !partnerName) return;

    setIsEncrypting(true);
    setProgress([]);
    setResult(null);

    const unlisten = await listen<EncryptProgress>("encrypt-progress", (event) => {
      setProgress((prev) => [...prev.slice(-100), event.payload]);
    });

    try {
      const res = await encryptBatch(selectedFiles, partnerName, certPaths, outputDir);
      setResult(res);
    } catch (e) {
      setResult({ total: 0, success_count: 0, error_count: 1, errors: [String(e)] });
    } finally {
      unlisten();
      setIsEncrypting(false);
    }
  }, [selectedFiles]);

  const reset = useCallback(() => {
    setProgress([]);
    setResult(null);
  }, []);

  return {
    selectedFiles,
    setSelectedFiles,
    selectedRecipientIds,
    setSelectedRecipientIds,
    isEncrypting,
    progress,
    result,
    startEncrypt,
    reset,
  };
}

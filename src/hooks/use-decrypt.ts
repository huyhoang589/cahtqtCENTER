import { useCallback, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { DecryptProgress, DecryptResult } from "../types";
import { decryptBatch } from "../lib/tauri-api";

export function useDecrypt() {
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [selectedPartnerName, setSelectedPartnerName] = useState<string | null>(null);
  const [isDecrypting, setIsDecrypting] = useState(false);
  const [progress, setProgress] = useState<DecryptProgress[]>([]);
  const [result, setResult] = useState<DecryptResult | null>(null);

  /// PIN read from AppState.token_login — must be LoggedIn before calling
  const startDecrypt = useCallback(async (outputDir?: string | null) => {
    if (selectedFiles.length === 0 || !selectedPartnerName) return;

    setIsDecrypting(true);
    setProgress([]);
    setResult(null);

    const unlisten = await listen<DecryptProgress>("decrypt-progress", (event) => {
      setProgress((prev) => [...prev.slice(-100), event.payload]);
    });

    try {
      const res = await decryptBatch(selectedFiles, selectedPartnerName, outputDir);
      setResult(res);
    } catch (e) {
      setResult({ total: 0, success_count: 0, error_count: 1, errors: [String(e)] });
    } finally {
      unlisten();
      setIsDecrypting(false);
    }
  }, [selectedFiles, selectedPartnerName]);

  const reset = useCallback(() => {
    setProgress([]);
    setResult(null);
  }, []);

  return {
    selectedFiles,
    setSelectedFiles,
    selectedPartnerName,
    setSelectedPartnerName,
    isDecrypting,
    progress,
    result,
    startDecrypt,
    reset,
  };
}

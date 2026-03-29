import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AppLogPayload, EncryptProgress, DecryptProgress } from "../types";

export interface LogEntry {
  id: string;
  timestamp: string;
  level: "info" | "success" | "warning" | "error";
  message: string;
}

const MAX_ENTRIES = 200;

// D: suppress noisy settings-persistence logs from the log panel
const FILTERED_PATTERNS = [
  "encrypt_panel_split_ratio",
  "decrypt_panel_split_ratio",
];

export function useLogPanel() {
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const idRef = useRef(0);

  const addEntry = useCallback((level: LogEntry["level"], message: string, ts?: string) => {
    if (FILTERED_PATTERNS.some((p) => message.includes(p))) return;  // D: drop noisy settings logs
    const timestamp = ts ?? new Date().toTimeString().slice(0, 8);
    setEntries((prev) => {
      const next = [...prev, { id: String(++idRef.current), timestamp, level, message }];
      return next.length > MAX_ENTRIES ? next.slice(-MAX_ENTRIES) : next;
    });
  }, []);

  const clearEntries = useCallback(() => setEntries([]), []);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    listen<EncryptProgress>("encrypt-progress", (e) => {
      const p = e.payload;
      const lvl = p.status === "error" ? "error" : p.status === "success" ? "success" : "info";
      addEntry(lvl, `[Encrypt] ${p.file_name} (${p.current}/${p.total}) ${p.status}`);
    }).then((fn) => unlisteners.push(fn)).catch(console.error);

    listen<DecryptProgress>("decrypt-progress", (e) => {
      const p = e.payload;
      const lvl = p.status === "error" ? "error" : p.status === "success" ? "success" : "info";
      addEntry(lvl, `[Decrypt] ${p.file_name} (${p.current}/${p.total}) ${p.status}`);
    }).then((fn) => unlisteners.push(fn)).catch(console.error);

    listen<AppLogPayload>("app_log", (e) => {
      addEntry(e.payload.level, e.payload.message, e.payload.timestamp);
    }).then((fn) => unlisteners.push(fn)).catch(console.error);

    return () => { unlisteners.forEach((fn) => fn()); };
  }, [addEntry]);

  return { entries, addEntry, clearEntries };
}

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface SettingsStore {
  outputDataDir: string;
  pkcs11Mode: "auto" | "manual";
  pkcs11ManualPath: string;
  isLoaded: boolean;
  loadSettings: () => Promise<void>;
  saveOutputDataDir: (dir: string) => Promise<void>;
  savePkcs11Mode: (mode: "auto" | "manual") => Promise<void>;
  savePkcs11ManualPath: (path: string) => Promise<void>;
}

export function useSettingsStore(): SettingsStore {
  const [outputDataDir, setOutputDataDir] = useState("");
  const [pkcs11Mode, setPkcs11Mode] = useState<"auto" | "manual">("auto");
  const [pkcs11ManualPath, setPkcs11ManualPath] = useState("");
  const [isLoaded, setIsLoaded] = useState(false);

  const loadSettings = async () => {
    const settings = await invoke<Record<string, string>>("get_settings");
    setOutputDataDir(settings["output_data_dir"] ?? "");
    setPkcs11Mode((settings["pkcs11_mode"] as "auto" | "manual") ?? "auto");
    setPkcs11ManualPath(settings["pkcs11_manual_path"] ?? "");
    setIsLoaded(true);
  };

  useEffect(() => { loadSettings().catch(() => {}); }, []);

  const saveOutputDataDir = async (dir: string) => {
    setOutputDataDir(dir);
    await invoke("set_setting", { key: "output_data_dir", value: dir });
  };

  const savePkcs11Mode = async (mode: "auto" | "manual") => {
    setPkcs11Mode(mode);
    await invoke("set_setting", { key: "pkcs11_mode", value: mode });
  };

  const savePkcs11ManualPath = async (path: string) => {
    setPkcs11ManualPath(path);
    await invoke("set_setting", { key: "pkcs11_manual_path", value: path });
  };

  return {
    outputDataDir, pkcs11Mode, pkcs11ManualPath, isLoaded,
    loadSettings, saveOutputDataDir, savePkcs11Mode, savePkcs11ManualPath,
  };
}

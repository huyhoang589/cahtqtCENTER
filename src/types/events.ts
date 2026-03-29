// Tauri event payload types — emitted via app_log and progress channels

export interface EncryptProgress {
  current: number;
  total: number;
  file_name: string;
  file_path: string;
  status: "processing" | "success" | "warning" | "error";
  error: string | null;
}

export interface DecryptProgress {
  current: number;
  total: number;
  file_name: string;
  file_path: string;
  status: "processing" | "success" | "error";
  error: string | null;
}

export interface AppLogPayload {
  level: "info" | "success" | "warning" | "error";
  message: string;
  timestamp: string;
}

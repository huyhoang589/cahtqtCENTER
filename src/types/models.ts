// Domain model types — DB entities and API response shapes

export interface Partner {
  id: string;
  name: string;
  created_at: number;
  member_count?: number;
}

export interface PartnerMember {
  id: string;
  partner_id: string;
  name: string;
  email: string | null;
  cert_cn: string;
  cert_serial: string;
  cert_valid_from: number;
  cert_valid_to: number;
  cert_file_path: string;
  cert_org: string | null;
  created_at: number;
}

export interface CertInfo {
  cn: string;
  email: string | null;
  org: string | null;
  serial: string;
  valid_from: number;
  valid_to: number;
  issuer_cn?: string | null;
  file_path?: string | null;
}

export interface EncLog {
  id: string;
  operation: string;
  src_file: string;
  dst_file: string;
  partner_member_id: string | null;
  status: string;
  error_msg: string | null;
  created_at: number;
}

export interface Setting {
  key: string;
  value: string;
}

export type FileStatus = "pending" | "encrypting" | "decrypting" | "done" | "warning" | "error";

export interface EncryptResult {
  total: number;
  success_count: number;
  error_count: number;
  errors: string[];
}

export interface DecryptResult {
  total: number;
  success_count: number;
  error_count: number;
  errors: string[];
}

export interface AppInfo {
  version: string;
  app_data_dir: string;
  dll_loaded: boolean;
}

export interface AppSettings {
  output_data_dir: string;
  pkcs11_mode: string;
  pkcs11_manual_path: string;
}

// ---- eToken Module Types -------------------------------------------------------

export interface LibraryInfo {
  vendor: string;
  description: string;
  path: string;
  cryptoki_version: string;
  library_version: string;
  manufacturer_id: string;
}

export interface SlotInfo {
  slot_id: number;
  slot_description: string;
  manufacturer: string;
  hardware_version: string;
  firmware_version: string;
  token_present: boolean;
}

export interface TokenInfo {
  slot_id: number;
  label: string;
  manufacturer: string;
  model: string;
  serial_number: string;
  firmware_version: string;
  pin_min_len: number;
  pin_max_len: number;
  pin_initialized: boolean;
  user_pin_locked: boolean;
  user_pin_final_try: boolean;
  user_pin_count_low: boolean;
}

export interface CertificateInfo {
  object_id: string;       // hex CKA_ID — used as unique key
  label: string;           // CKA_LABEL
  subject_cn: string;
  subject_email: string;
  subject_org: string;
  subject_unit: string;
  issuer_cn: string;
  issuer_org: string;
  serial_number: string;
  valid_from: string;      // "YYYY-MM-DD"
  valid_until: string;     // "YYYY-MM-DD"
  is_expired: boolean;
  is_ca: boolean;
  key_usage: string[];
  fingerprint_sha1: string;
  // raw_der intentionally omitted (serde skip on Rust side)
}

export interface TokenCertEntry {
  slot_id: number;
  certificate: CertificateInfo;
}

export interface MechanismDetail {
  name: string;            // "RSA_PKCS_OAEP" | "RSA_PKCS_PSS"
  pkcs_standard: string;   // "PKCS#1 v2.1"
  min_key_bits: number;
  max_key_bits: number;
  flags: string[];         // ["encrypt", "decrypt", "wrap"] etc.
  supported: boolean;
}

export interface TokenScanResult {
  library: LibraryInfo;
  slots: SlotInfo[];
  tokens: TokenInfo[];
  certificates: TokenCertEntry[];
  mechanisms: MechanismDetail[];
  scan_time: string;
  error: string | null;
}

export interface SenderCertExportResult {
  saved_path: string;
  display_name: string;
  email: string;
  organization: string;
  serial: string;
  valid_until: string;
}

export type ScanStatus =
  | { type: "idle" }
  | { type: "scanning" }
  | { type: "done"; found: boolean }
  | { type: "error"; message: string };

// ---- Token 3-state status ------------------------------------------------------

export type TokenStatus = "disconnected" | "connected" | "logged_in";

export interface TokenStatusResponse {
  status: TokenStatus;
  cert_cn: string | null;
  dll_found: boolean;
}

export interface LoginTokenResult {
  cert_cn: string;
  status: string; // "logged_in"
}

// ---- Cert Expiry ---------------------------------------------------------------

export type CertExpiryStatus = "valid" | "expiring_soon" | "expired";

export function getCertExpiryStatus(valid_to: number): CertExpiryStatus {
  const now = Math.floor(Date.now() / 1000);
  const thirtyDays = 30 * 24 * 60 * 60;
  if (valid_to < now) return "expired";
  if (valid_to < now + thirtyDays) return "expiring_soon";
  return "valid";
}

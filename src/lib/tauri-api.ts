import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import type {
  AppInfo,
  AppSettings,
  CertInfo,
  CredentialPreview,
  DecryptResult,
  EncLog,
  EncryptResult,
  GenerateLicenseResult,
  LicenseAuditEntry,
  LibraryInfo,
  LoginTokenResult,
  MachineCredential,
  Partner,
  PartnerMember,
  SenderCertExportResult,
  TokenScanResult,
  TokenStatusResponse,
} from "../types";

// ---- Settings ----------------------------------------------------------------

export const getSettings = () =>
  invoke<Record<string, string>>("get_settings");

export const setSetting = (key: string, value: string) =>
  invoke<void>("set_setting", { key, value });

export const getAppInfo = () => invoke<AppInfo>("get_app_info");

export const getAppSettings = () => invoke<AppSettings>("get_app_settings");

export const openFolder = (path: string) => invoke<void>("open_folder", { path });

// C-2: pass cert metadata so Rust can build {CN}-{Serial}.crt filename
export const exportMemberCert = (certPath: string, destDir: string, certCn?: string, certSerial?: string) =>
  invoke<string>("export_member_cert", { certPath, destDir, certCn: certCn ?? null, certSerial: certSerial ?? null });

export const importSenderCert = (certPath: string) =>
  invoke<CertInfo>("import_sender_cert", { certPath });

// ---- eToken Commands -----------------------------------------------------------

/** Full token scan. Pass libPathOverride to use a custom library path. */
export const tokenScan = (libPathOverride?: string) =>
  invoke<TokenScanResult>("token_scan", { libPathOverride: libPathOverride ?? null });

/** Quick library detection — no cert reading. Used on Settings page load. */
export const tokenGetLibraryInfo = () =>
  invoke<LibraryInfo>("token_get_library_info");

/** Export selected cert as sender cert. Reads raw DER from AppState cache. */
export const tokenExportSenderCert = (certObjectId: string, slotId: number) =>
  invoke<SenderCertExportResult>("token_export_sender_cert", { certObjectId, slotId });

/** Set custom PKCS#11 library path. Validates + saves to settings. */
export const tokenSetLibraryPath = (path: string) =>
  invoke<LibraryInfo>("token_set_library_path", { path });

/** Clear sender cert from settings (does NOT delete file from disk). */
export const tokenClearSenderCert = () =>
  invoke<void>("token_clear_sender_cert");

/** Login to token via PKCS#11 C_Login. Stores verified state in Rust AppState. */
export const loginToken = (pin: string) =>
  invoke<LoginTokenResult>("login_token", { pin });

/** Logout from token — zeroizes PIN from AppState. */
export const logoutToken = () => invoke<void>("logout_token");

/** Get current token status for UI polling. */
export const getTokenStatus = () =>
  invoke<TokenStatusResponse>("get_token_status");

// ---- File / folder pickers (frontend dialog — no Rust command needed) --------

export const selectFiles = async (
  filters?: { name: string; extensions: string[] }[]
): Promise<string[] | null> => {
  const result = await open({ multiple: true, filters: filters ?? [] });
  if (!result) return null;
  return Array.isArray(result) ? result : [result];
};

export const selectAllFiles = () => selectFiles([]);

export const selectCertFile = () =>
  selectFiles([
    { name: "Certificates", extensions: ["crt", "pem", "cer", "der"] },
  ]);

export const selectDllFile = () =>
  selectFiles([{ name: "DLL Files", extensions: ["dll"] }]);

export const selectOutputDir = async (): Promise<string | null> => {
  const result = await open({ directory: true, multiple: false });
  return typeof result === "string" ? result : null;
};

// ---- Partners (formerly Groups) ----------------------------------------------

export const createPartner = (name: string) =>
  invoke<Partner>("create_partner", { name });

export const listPartners = () => invoke<Partner[]>("list_partners");

export const renamePartner = (id: string, name: string) =>
  invoke<void>("rename_partner", { id, name });

export const deletePartner = (id: string) =>
  invoke<void>("delete_partner", { id });

// ---- Partner Members (formerly Recipients) -----------------------------------

export const importCertPreview = (certPath: string) =>
  invoke<CertInfo>("import_cert_preview", { certPath });

export const addPartnerMember = (
  partnerId: string,
  certPath: string,
  name?: string,
  email?: string
) =>
  invoke<PartnerMember>("add_partner_member", {
    partnerId,
    certPath,
    name: name ?? null,
    email: email ?? null,
  });

export const listPartnerMembers = (partnerId: string) =>
  invoke<PartnerMember[]>("list_partner_members", { partnerId });

export const deletePartnerMember = (id: string) =>
  invoke<void>("delete_partner_member", { id });

// ---- Encrypt / Decrypt -------------------------------------------------------

export const encryptBatch = (
  srcPaths: string[],
  partnerName: string,
  certPaths: string[],
  outputDir?: string | null,
) =>
  invoke<EncryptResult>("encrypt_batch", { srcPaths, partnerName, certPaths, outputDir: outputDir ?? null });

export const decryptBatch = (
  filePaths: string[],
  partnerName: string,
  outputDir?: string | null,
) =>
  invoke<DecryptResult>("decrypt_batch", { filePaths, partnerName, outputDir: outputDir ?? null });

export const setCommunication = (
  recipientCertPath: string,
  partnerName: string,
  destDir: string,
  pin: string,
) =>
  invoke<string>("set_communication", { recipientCertPath, partnerName, destDir, pin });

// ---- Logs --------------------------------------------------------------------

export const listLogs = (limit: number, offset: number) =>
  invoke<EncLog[]>("list_logs", { limit, offset });

// ---- License Gen -------------------------------------------------------------

export const selectCredentialFile = () =>
  selectFiles([{ name: "JSON Files", extensions: ["json"] }]);

export const importCredential = (path: string) =>
  invoke<CredentialPreview>("import_credential", { path });

export const generateLicense = (
  credential: MachineCredential,
  expiresAt: number | null,
  unitName: string,
) =>
  invoke<GenerateLicenseResult>("generate_license", { credential, expiresAt, unitName });

export const listLicenseAudit = (limit: number, offset: number) =>
  invoke<LicenseAuditEntry[]>("list_license_audit", { limit, offset });

export const exportLicense = (auditId: string) =>
  invoke<string>("export_license", { auditId });

export const deleteLicense = (auditId: string) =>
  invoke<void>("delete_license", { auditId });

export const openLicenseFolder = (userName: string) =>
  invoke<void>("open_license_folder", { userName });

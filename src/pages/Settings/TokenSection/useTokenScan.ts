import { useCallback, useEffect, useState } from "react";
import {
  getSettings,
  tokenClearSenderCert,
  tokenExportSenderCert,
  tokenScan,
} from "../../../lib/tauri-api";
import type {
  LibraryInfo,
  ScanStatus,
  SenderCertExportResult,
  TokenScanResult,
} from "../../../types";

// Props passed from SettingsPage via TokenSection
export type Pkcs11Mode = "auto" | "manual";

interface SelectedCert {
  certObjectId: string;
  slotId: number;
}

export interface UseTokenScanReturn {
  scanStatus: ScanStatus;
  scanResult: TokenScanResult | null;
  selectedCert: SelectedCert | null;
  senderCert: SenderCertExportResult | null;
  libraryInfo: LibraryInfo | null;
  error: string | null;
  scan: () => Promise<void>;
  selectCert: (certObjectId: string, slotId: number) => void;
  exportAsSender: () => Promise<void>;
  clearSender: () => Promise<void>;
}

export function useTokenScan(pkcs11Mode: Pkcs11Mode = "auto", pkcs11ManualPath = ""): UseTokenScanReturn {
  const [scanStatus, setScanStatus] = useState<ScanStatus>({ type: "idle" });
  const [scanResult, setScanResult] = useState<TokenScanResult | null>(null);
  const [selectedCert, setSelectedCert] = useState<SelectedCert | null>(null);
  const [senderCert, setSenderCert] = useState<SenderCertExportResult | null>(null);
  const [libraryInfo, setLibraryInfo] = useState<LibraryInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  // On mount: restore saved sender cert from settings
  useEffect(() => {
    getSettings()
      .then((s) => {
        const cn = s["sender_cn"];
        if (cn) {
          setSenderCert({
            display_name: cn,
            email: s["sender_email"] ?? "",
            organization: s["sender_org"] ?? "",
            serial: s["sender_serial"] ?? "",
            valid_until: s["sender_valid_until"] ?? "",
            saved_path: s["sender_cert_path"] ?? "",
          });
        }
      })
      .catch(() => {});
  }, []);

  const scan = useCallback(async () => {
    setScanStatus({ type: "scanning" });
    setError(null);
    setScanResult(null);
    setSelectedCert(null);
    // Use manual path as override when in manual mode
    const libOverride = pkcs11Mode === "manual" && pkcs11ManualPath ? pkcs11ManualPath : undefined;
    try {
      const result = await tokenScan(libOverride);
      setScanResult(result);
      setLibraryInfo(result.library);
      setScanStatus({ type: "done", found: result.certificates.length > 0 });
    } catch (e) {
      const msg = String(e);
      setError(msg);
      setScanStatus({ type: "error", message: msg });
    }
  }, [pkcs11Mode, pkcs11ManualPath]);

  const selectCert = useCallback((certObjectId: string, slotId: number) => {
    setSelectedCert({ certObjectId, slotId });
  }, []);

  const exportAsSender = useCallback(async () => {
    if (!selectedCert) return;
    try {
      const result = await tokenExportSenderCert(
        selectedCert.certObjectId,
        selectedCert.slotId,
      );
      setSenderCert(result);
    } catch (e) {
      setError(`Export failed: ${e}`);
    }
  }, [selectedCert]);

  const clearSender = useCallback(async () => {
    await tokenClearSenderCert();
    setSenderCert(null);
  }, []);

  return {
    scanStatus,
    scanResult,
    selectedCert,
    senderCert,
    libraryInfo,
    error,
    scan,
    selectCert,
    exportAsSender,
    clearSender,
  };
}

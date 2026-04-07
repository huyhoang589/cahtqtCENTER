import { useCallback, useEffect, useState } from "react";
import type {
  CredentialPreview,
  GenerateLicenseResult,
  LicenseAuditEntry,
} from "../types";
import {
  generateLicense,
  importCredential,
  listLicenseAudit,
  selectCredentialFile,
} from "../lib/tauri-api";

export function useLicenseGen() {
  const [credential, setCredential] = useState<CredentialPreview | null>(null);
  const [expiresAt, setExpiresAt] = useState<number | null>(() => {
    // Default: 1 year from now
    return Math.floor(Date.now() / 1000) + 365 * 24 * 60 * 60;
  });
  const [isPerpetual, setIsPerpetual] = useState(false);
  const [unitName, setUnitName] = useState("Default");
  const [isGenerating, setIsGenerating] = useState(false);
  const [result, setResult] = useState<GenerateLicenseResult | null>(null);
  const [auditEntries, setAuditEntries] = useState<LicenseAuditEntry[]>([]);

  const loadAuditHistory = useCallback(async () => {
    try {
      const entries = await listLicenseAudit(50, 0);
      setAuditEntries(entries);
    } catch {
      // Silently fail — table may not exist yet on fresh DB
    }
  }, []);

  // Load history on mount
  useEffect(() => {
    loadAuditHistory();
  }, [loadAuditHistory]);

  const handleImport = useCallback(async () => {
    const files = await selectCredentialFile();
    if (!files || files.length === 0) return;
    try {
      const preview = await importCredential(files[0]);
      setCredential(preview);
      setResult(null);
    } catch (e) {
      setCredential(null);
      setResult({ success: false, outputPath: "", machineFp: "", error: String(e) });
    }
  }, []);

  const handleGenerate = useCallback(async () => {
    if (!credential?.credential) return;
    setIsGenerating(true);
    setResult(null);
    try {
      const res = await generateLicense(
        credential.credential,
        isPerpetual ? null : expiresAt,
        unitName,
      );
      setResult(res);
      await loadAuditHistory();
    } catch (e) {
      setResult({ success: false, outputPath: "", machineFp: "", error: String(e) });
    } finally {
      setIsGenerating(false);
    }
  }, [credential, expiresAt, isPerpetual, unitName, loadAuditHistory]);

  return {
    credential,
    expiresAt,
    setExpiresAt,
    isPerpetual,
    setIsPerpetual,
    unitName,
    setUnitName,
    isGenerating,
    result,
    auditEntries,
    handleImport,
    handleGenerate,
    loadAuditHistory,
  };
}

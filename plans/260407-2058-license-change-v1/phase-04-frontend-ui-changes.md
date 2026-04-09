# Phase 4: Frontend UI Changes

## Context
- [Brainstorm](../../plans/reports/brainstorm-260407-2058-license-change-v1.md)
- [LicenseGenPage.tsx](../../src/pages/LicenseGenPage.tsx)
- [use-license-gen.ts](../../src/hooks/use-license-gen.ts)
- [tauri-api.ts](../../src/lib/tauri-api.ts) — lines 170-186
- [models.ts](../../src/types/models.ts) — lines 189-233

## Overview
- **Priority:** P1
- **Status:** Complete
- Split preview into 2 cards (credential left, license payload right)
- Unmask token serial, black text in cards
- Add export/delete buttons to audit history
- Add confirmation dialog for delete

## Related Code Files

### Modify
- `src/pages/LicenseGenPage.tsx`
- `src/hooks/use-license-gen.ts`
- `src/lib/tauri-api.ts`
- `src/types/models.ts`

## Implementation Steps

### 1. Update TypeScript types (`models.ts`)

Add `licenseBlob` to `LicenseAuditEntry`:
```typescript
export interface LicenseAuditEntry {
  // ... existing fields ...
  licenseBlob: string | null; // NEW
}
```

### 2. Add API functions (`tauri-api.ts`)

```typescript
export const exportLicense = (auditId: string) =>
  invoke<string>("export_license", { auditId });

export const deleteLicense = (auditId: string) =>
  invoke<void>("delete_license", { auditId });
```

### 3. Update hook (`use-license-gen.ts`)

Add `handleExport` and `handleDelete` callbacks:
```typescript
const handleExport = useCallback(async (id: string) => {
  try {
    const path = await exportLicense(id);
    setResult({ success: true, outputPath: path, machineFp: "", error: null });
  } catch (e) {
    setResult({ success: false, outputPath: "", machineFp: "", error: String(e) });
  }
}, []);

const handleDelete = useCallback(async (id: string) => {
  try {
    await deleteLicense(id);
    await loadAuditHistory();
    setResult(null);
  } catch (e) {
    setResult({ success: false, outputPath: "", machineFp: "", error: String(e) });
  }
}, [loadAuditHistory]);
```

Add new state for `lastGeneratedPayload` to populate the right card:
```typescript
const [lastPayload, setLastPayload] = useState<GenerateLicenseResult | null>(null);
```

Update `handleGenerate` to store the result for display in right card.

Add `handleOpenFolder` callback — invokes a Tauri `open_license_folder` command or uses `shell.open` to open the LICENSE directory for the given user name.

Return `handleExport`, `handleDelete`, `handleOpenFolder`, `lastPayload` from hook.

### 4. Rewrite preview section in `LicenseGenPage.tsx`

Replace the single "Credential Preview" card (lines 59-81) with two side-by-side cards:

**Layout:**
```tsx
<div style={{ display: "flex", gap: 16 }}>
  {/* Left: Credential Preview */}
  <div style={{ flex: 1, border: "1px solid var(--color-border-light)", borderRadius: "var(--radius-md)", padding: 16 }}>
    <div style={{ fontWeight: "bold", marginBottom: 8, color: "#000" }}>Credential Preview</div>
    {/* Grid: User, Token Serial (FULL), CPU ID, Board Serial, Registered At */}
    {/* All text color: #000 */}
  </div>

  {/* Right: License Preview (only after generation) */}
  <div style={{ flex: 1, border: "1px solid var(--color-border-light)", borderRadius: "var(--radius-md)", padding: 16 }}>
    <div style={{ fontWeight: "bold", marginBottom: 8, color: "#000" }}>License Preview</div>
    {/* Grid: Product, Issued By, Issued At, Machine FP, Expires At, Token Serial */}
    {/* All text color: #000 */}
    {/* Show placeholder "Generate a license to preview" if no result */}
  </div>
</div>
```

Key changes:
- Remove `maskSerial()` usage for credential preview card
- All `<span>` text inside both cards: `color: "#000"`
- Labels still use slightly lighter color for distinction but **black** (`color: "#333"` or similar)
- Right card shows payload fields from `GenerateLicenseResult` + stored metadata

### 5. Update audit history table

Add "Actions" column with Export and Delete buttons:

```tsx
{/* Table header */}
{["Date", "User", "Unit", "Token", "Machine FP", "Expiry", "Actions"].map(...)}

{/* Table body — per row */}
<td style={{ padding: "6px 10px", display: "flex", gap: 4 }}>
  <!-- Updated: Validation Session 1 - Add Open Folder button next to Export -->
  <button
    className="btn btn-ghost"
    style={{ fontSize: "var(--font-size-xs)", padding: "2px 6px" }}
    onClick={() => handleExport(e.id)}
    disabled={!e.licenseBlob}
    title={!e.licenseBlob ? "No license data stored" : "Export to file"}
  >
    Export
  </button>
  <button
    className="btn btn-ghost"
    style={{ fontSize: "var(--font-size-xs)", padding: "2px 6px" }}
    onClick={() => handleOpenFolder(e.userName)}
    title="Open LICENSE folder"
  >
    Open Folder
  </button>
  <button
    className="btn btn-ghost"
    style={{ fontSize: "var(--font-size-xs)", padding: "2px 6px", color: "var(--color-error, #dc2626)" }}
    onClick={() => {
      if (window.confirm(`Delete license for ${e.userName}?`)) {
        handleDelete(e.id);
      }
    }}
  >
    Delete
  </button>
</td>
```

- Export button disabled when `licenseBlob` is null (old records)
- Delete shows `window.confirm()` dialog before proceeding

### 6. Remove `maskSerial` function

Remove the `maskSerial` helper (line 30-31) and all usages. Show full token serial everywhere.

### 7. Update `LicenseAuditEntry` in command response

In `src-tauri/src/commands/license_gen.rs`, add `license_blob` to `LicenseAuditEntry` struct and include it in the `list_license_audit` mapping.

## Todo
- [x] Add `licenseBlob` to TS `LicenseAuditEntry` type
- [x] Add `exportLicense` and `deleteLicense` API functions
- [x] Add `handleExport` and `handleDelete` to hook
- [x] Split preview into 2 side-by-side cards
- [x] Remove `maskSerial`, show full token serial
- [x] Black text in preview cards
- [x] Right card: show payload fields after generation
- [x] Add Actions column with Export/Delete/Open Folder buttons
- [x] Export disabled for null blob rows
- [x] Open Folder button opens LICENSE directory for user
- [x] Delete with `window.confirm()` dialog
- [x] Add `license_blob` to Rust `LicenseAuditEntry` + mapping
- [x] Run `tsc --noEmit` and `cargo check`

## Success Criteria
- Two preview cards side-by-side, all text black
- Token serial shown in full (no masking)
- Right card empty until license generated
- Export button saves file, shows success message
- Delete button removes from table + disk after confirmation
- Old audit rows show disabled Export button

## Risk
- `LicenseGenPage.tsx` is currently 132 lines. After changes it may approach ~200. If so, extract the two preview cards into a small component file.

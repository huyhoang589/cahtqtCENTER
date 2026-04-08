# Phase 4 — Frontend Page + Routing

## Context Links
- Existing page pattern: `src/pages/EncryptPage.tsx`, `src/pages/DecryptPage.tsx`
- Sidebar: `src/components/app-sidebar.tsx`
- Router: `src/App.tsx`
- Tauri API: `src/lib/tauri-api.ts`
- Types: `src/types/models.ts`, `src/types/index.ts`
- Token status context: `src/contexts/token-status-context.tsx`
- Brainstorm mockup: `plans/reports/brainstorm-260407-0833-license-gen-page.md`

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Create LicenseGenPage React component with credential import, preview, signing, and audit history table. Wire into routing and sidebar.

## Key Insights
- Existing pages use inline styles with CSS variables (no CSS modules or Tailwind)
- Icons from `lucide-react`
- Tauri API wrapper in `src/lib/tauri-api.ts` — add new functions there
- Token login status available via `useTokenStatus()` context hook
- File picker via `@tauri-apps/plugin-dialog` `open()` — already used in tauri-api.ts
- Pages are PascalCase files: `EncryptPage.tsx`, `DecryptPage.tsx`

## Requirements

### Functional
- Sidebar entry "License Gen" with icon (e.g. `Shield` or `FileKey` from lucide-react)
- Route `/license-gen` mapped to `LicenseGenPage`
- Import credential button → file picker (JSON files)
- Credential preview card showing: user_name, token_serial (masked), cpu_id, board_serial, machine_fp (computed)
- Expiry controls: date input (default +1 year) + perpetual toggle
- Generate License button (disabled if: no credential loaded, or token not logged in)
- Success/error status message after generation
- License History table showing recent audit entries (paginated)

### Non-functional
- Consistent with existing page styling (CSS vars, inline styles)
- Responsive within app shell constraints
- Token warning bar visible if token not logged in (existing component)

## Architecture

### Component Structure
```
src/pages/LicenseGenPage.tsx    — Main page component (~150-180 lines)
src/hooks/use-license-gen.ts    — Hook for credential state + generate logic
```

### State Flow
```
[Import Button] → selectCredentialFile() → importCredential(path)
                                         → setCredential(preview)
                                         → compute display fields

[Generate Button] → generateLicense(credential, expiresAt, unitName)
                  → setResult(success/error)
                  → refreshAuditHistory()

[History Table] ← listLicenseAudit(limit, offset) on mount + after generate
```

## Related Code Files

### Create
- `src/pages/LicenseGenPage.tsx`
- `src/hooks/use-license-gen.ts`

### Modify
- `src/App.tsx` — add Route for `/license-gen`
- `src/components/app-sidebar.tsx` — add nav item
- `src/lib/tauri-api.ts` — add API functions
- `src/types/models.ts` — add TypeScript types

## Implementation Steps

### Step 1: Add TypeScript types to `src/types/models.ts`
```typescript
export interface MachineCredential {
  token_serial: string;
  cpu_id: string;
  board_serial: string;
  user_name: string;
  registered_at: string;
}

export interface CredentialPreview {
  credential: MachineCredential;
  machine_fp: string;
  is_valid: boolean;
  validation_error: string | null;
}

export interface GenerateLicenseResult {
  success: boolean;
  output_path: string;
  machine_fp: string;
  error: string | null;
}

export interface LicenseAuditEntry {
  id: string;
  user_name: string;
  unit_name: string;
  token_serial: string;
  machine_fp: string;
  product: string;
  expires_at: number | null;
  created_at: number;
}
```

### Step 2: Add Tauri API functions to `src/lib/tauri-api.ts`
```typescript
// ---- License Gen -------------------------------------------------------

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
```

### Step 3: Create `src/hooks/use-license-gen.ts`

Custom hook managing:
- `credential: CredentialPreview | null` — imported credential state
- `expiresAt: number | null` — expiry timestamp (default: now + 1 year)
- `isPerpetual: boolean` — toggle for perpetual license
- `isGenerating: boolean` — loading state during sign operation
- `result: GenerateLicenseResult | null` — last generation result
- `auditEntries: LicenseAuditEntry[]` — history list
- `handleImport()` — trigger file picker + importCredential
- `handleGenerate()` — call generateLicense + refresh audit
- `loadAuditHistory()` — fetch from DB

### Step 4: Create `src/pages/LicenseGenPage.tsx`

Layout (following existing page patterns):
```
<div style={{ padding, display: flex, flexDirection: column, gap }}>
  {/* Header */}
  <h2>License Gen</h2>

  {/* Import section */}
  <button onClick={handleImport}>Import Credential File...</button>

  {/* Credential Preview Card (if credential loaded) */}
  <div style={{ border, borderRadius, padding }}>
    User: {credential.user_name}
    Token: {masked_serial}
    CPU ID: {credential.cpu_id}
    Board: {credential.board_serial}
    Machine FP: {machine_fp} (computed)
  </div>

  {/* Expiry controls */}
  <div>
    <input type="date" value={expiryDate} onChange={...} disabled={isPerpetual} />
    <label><input type="checkbox" checked={isPerpetual} /> Perpetual</label>
  </div>

  {/* Generate button */}
  <button onClick={handleGenerate} disabled={!credential || !tokenLoggedIn || isGenerating}>
    Generate License
  </button>

  {/* Status message */}
  {result && <div style={{ color: result.success ? 'green' : 'red' }}>
    {result.success ? `License saved to ${result.output_path}` : result.error}
  </div>}

  {/* Audit History Table */}
  <h3>License History</h3>
  <table>
    <thead><tr><th>Date</th><th>User</th><th>Token</th><th>Machine FP</th><th>Expiry</th></tr></thead>
    <tbody>{auditEntries.map(entry => <tr>...</tr>)}</tbody>
  </table>
</div>
```

### Step 5: Add route to `src/App.tsx`
```tsx
import LicenseGenPage from "./pages/LicenseGenPage";
// In Routes:
<Route path="/license-gen" element={<LicenseGenPage />} />
```

### Step 6: Add sidebar item to `src/components/app-sidebar.tsx`
```tsx
import { FileKey } from "lucide-react";
// Add to NAV_ITEMS array:
{ to: "/license-gen", label: "License Gen", Icon: FileKey },
```

### Step 7: Run `npm run lint` and fix any TypeScript errors

## Todo List
- [x] Add TS types to `src/types/models.ts`
- [x] Add API functions to `src/lib/tauri-api.ts`
- [x] Create `src/hooks/use-license-gen.ts` hook
- [x] Create `src/pages/LicenseGenPage.tsx` page component
- [x] Add route in `src/App.tsx`
- [x] Add sidebar nav item in `src/components/app-sidebar.tsx`
- [x] Run `npm run lint` — fix errors
- [x] Manual UI test: page loads, file picker opens, history table renders

## Success Criteria
- Page accessible at `/license-gen` route
- Sidebar shows "License Gen" with icon
- File picker filters for `.json` files
- Credential preview displays correctly after import
- Generate button disabled when no credential or token not logged in
- Audit history table loads on mount
- No TypeScript compilation errors

## Risk Assessment
- **LOW**: Inline styles may grow verbose — keep component under 200 lines by extracting hook logic
- **LOW**: Date handling for expiry — use native HTML date input + convert to Unix timestamp

## Security Considerations
- Token serial masked in UI (show first 4 chars + `****`)
- No sensitive data (PINs, private keys) in frontend state
- Credential file path not exposed in audit UI

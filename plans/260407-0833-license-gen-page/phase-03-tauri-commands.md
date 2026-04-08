# Phase 3 — Tauri Commands

## Context Links
- Existing command pattern: `src-tauri/src/commands/encrypt.rs`
- Token login state: `src-tauri/src/etoken/models.rs` → `TokenLoginState`
- Lock helper: `src-tauri/src/lock_helper.rs`
- Command registration: `src-tauri/src/lib.rs` → `invoke_handler`

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Create Tauri command handlers bridging frontend to license_gen core module + audit repo.

## Key Insights
- Follow existing `encrypt_batch` pattern: check token login → extract PIN/slot → open session → execute → return result
- Three commands needed: `import_credential`, `generate_license`, `list_license_audit`
- `import_credential` is lightweight (parse JSON file, validate, return preview)
- `generate_license` needs `is_operation_running` guard to avoid token session conflicts
- Reuse `open_token_session` from `htqt_ffi/token_context.rs` but lighter — no DLL callbacks needed

## Requirements

### Functional
- `import_credential(path)` — Read JSON file, validate, return parsed credential + computed machine_fp
- `generate_license(credential, expires_at, unit_name)` — Full signing flow, write license.dat, insert audit
- `list_license_audit(limit, offset)` — Query audit table for history display

### Non-functional
- `generate_license` must acquire `is_operation_running` guard
- Error messages user-friendly (same style as encrypt errors)

## Architecture

### Command API

```typescript
// Frontend → Backend
import_credential(path: string) → CredentialPreview
generate_license(credential: MachineCredential, expiresAt: number | null, unitName: string) → GenerateLicenseResult
list_license_audit(limit: number, offset: number) → LicenseAuditEntry[]
```

### Types

```rust
// CredentialPreview — returned by import_credential
struct CredentialPreview {
    credential: MachineCredential,
    machine_fp: String,           // computed on import
    is_valid: bool,
    validation_error: Option<String>,
}

// GenerateLicenseResult — returned by generate_license
struct GenerateLicenseResult {
    success: bool,
    output_path: String,
    machine_fp: String,
    error: Option<String>,
}

// LicenseAuditEntry — from DB query (matches LicenseAuditRow + formatted fields)
struct LicenseAuditEntry {
    id: String,
    user_name: String,
    unit_name: String,
    token_serial: String,
    machine_fp: String,
    product: String,
    expires_at: Option<i64>,
    created_at: i64,
}
```

## Related Code Files

### Create
- `src-tauri/src/commands/license_gen.rs`

### Modify
- `src-tauri/src/commands/mod.rs` — add `pub mod license_gen;`
- `src-tauri/src/lib.rs` — register commands in `invoke_handler`

## Implementation Steps

### Step 1: Create `src-tauri/src/commands/license_gen.rs`

#### `import_credential` command
```rust
#[tauri::command]
pub async fn import_credential(path: String) -> Result<CredentialPreview, String> {
    // 1. Read file contents
    // 2. Parse JSON into MachineCredential
    // 3. Validate with license_gen::validate_credential()
    // 4. Compute machine_fp via payload::compute_machine_fp()
    // 5. Return CredentialPreview
}
```

#### `generate_license` command
```rust
#[tauri::command]
pub async fn generate_license(
    app: AppHandle,
    credential: MachineCredential,
    expires_at: Option<i64>,
    unit_name: String,
    state: State<'_, AppState>,
) -> Result<GenerateLicenseResult, String> {
    // 1. Acquire OperationGuard
    // 2. Extract token login state (pkcs11_lib, slot_id, pin)
    // 3. Validate credential
    // 4. Compute machine_fp
    // 5. Build LicensePayload
    // 6. Serialize to canonical JSON bytes
    // 7. spawn_blocking: open_token_session → sign_payload → assemble_license_dat
    //    NOTE: open_token_session needs simplified version — no DLL callbacks
    //    Use token_manager::initialize() + session + C_Sign directly
    // 8. Write license.dat to output_dir/LICENSE/{user_name}-license.dat
    // 9. Read server token serial from token_info for audit
    // 10. Insert audit record via license_audit_repo
    // 11. Return GenerateLicenseResult
}
```

**IMPLEMENTATION DECISION**: ~~Extract shared `open_pkcs11_session()` helper~~ **SIMPLIFIED** — generate_license command opens PKCS#11 session directly within the command handler. This is simpler than extracting a shared helper and avoids refactoring TokenContext.

**Rationale:**
- generate_license has simpler session lifecycle (open → sign → close)
- TokenContext needs to maintain session state for token login UI flow
- Refactoring TokenContext has higher risk of breaking existing code
- Direct session open in command is clear, easy to debug

#### `list_license_audit` command
```rust
#[tauri::command]
pub async fn list_license_audit(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>,
) -> Result<Vec<LicenseAuditEntry>, String> {
    license_audit_repo::list_audit(&state.db, limit, offset)
        .await
        .map_err(|e| e.to_string())
        // Map LicenseAuditRow → LicenseAuditEntry (same fields, direct mapping)
}
```

### Step 2: Add `pub mod license_gen;` to `src-tauri/src/commands/mod.rs`

### Step 3: Register commands in `src-tauri/src/lib.rs` invoke_handler:
```rust
commands::license_gen::import_credential,
commands::license_gen::generate_license,
commands::license_gen::list_license_audit,
```

### Step 4: Resolve output directory
- Use `output_dir::resolve_output_dir()` with subpath `"LICENSE"`
- Write file as `{user_name}-license.dat` (sanitize user_name for filesystem)

### Step 5: Read server token serial
- After opening PKCS#11 session, call `token_manager::get_token_infos()` to read server token serial
- Store in `LicensePayload.issued_by` and audit record

### Step 6: Run `cargo build` to verify compilation

## Todo List
- [x] Create `commands/license_gen.rs` with all 3 commands
- [x] Implement `import_credential` — file read + parse + validate + preview
- [x] Implement `generate_license` — full signing flow with operation guard
- [x] Implement `list_license_audit` — DB query wrapper
- [x] Add `pub mod license_gen;` to `commands/mod.rs`
- [x] Register 3 commands in `lib.rs` invoke_handler
- [x] Run `cargo build` to verify compilation

## Success Criteria
- All 3 commands compile and register in Tauri
- `import_credential` parses valid JSON and rejects invalid
- `generate_license` produces valid Base64 license.dat file
- `list_license_audit` returns audit rows ordered by created_at DESC
- Operation guard prevents concurrent license gen + encrypt/decrypt

## Risk Assessment
- **LOW** (validated): Direct session open in command handler. Simpler than extracting shared helper.
- **LOW**: Output path conflicts — use `{user_name}-license.dat` naming. Overwrite existing if regenerating for same user.

## Security Considerations
- PIN obtained from AppState.token_login — never stored in license_gen commands
- Operation guard prevents concurrent PKCS#11 sessions (token is single-threaded)
- Sanitize user_name before using in file paths (prevent path traversal)

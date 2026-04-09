# Code Review: License Gen Feature

**Date:** 2026-04-07  
**Reviewer:** code-reviewer  
**Branch:** feature/licenseGen  

## Scope
- **Files:** 14 (6 new Rust, 2 new TS/TSX, 6 modified)
- **LOC:** ~600 new lines
- **Focus:** Full-stack license generation: credential import, PKCS#11 signing, file output, audit trail

## Overall Assessment

Solid implementation that follows existing codebase patterns (OperationGuard, safe_lock, output_dir resolver, migration runner). Serde camelCase alignment is correct. PKCS#11 session lifecycle uses RAII properly. A few production-readiness issues found below.

---

## Critical Issues

### C1. Silently swallowed audit insert failure (commands/license_gen.rs:187)

```rust
let _ = license_audit_repo::insert_audit(
    &state.db,
    ...
).await;
```

The `let _` discards the Result. If the DB insert fails (disk full, schema mismatch, constraint violation), the license file was already written but no audit record exists. The user sees success with no indication that audit is missing.

**Impact:** Silent data loss in audit trail. Compliance gap if audit records are legally required.

**Fix:** At minimum, log the error. Better: return it as a warning in the response.

```rust
if let Err(e) = license_audit_repo::insert_audit(...).await {
    eprintln!("WARN: audit insert failed: {}", e);
    // Optionally: return GenerateLicenseResult with success=true but error=Some(warning)
}
```

### C2. Canonical JSON relies on struct field declaration order (payload.rs:14-25)

Comment says "struct fields in alphabetical order so serde serializes sorted keys by default." While serde's derive macro does serialize fields in declaration order (which happens to be alphabetical here), this is an **undocumented implementation detail**, not a guarantee. A future field addition out of alphabetical order or a serde version change could silently break signature verification.

**Impact:** If field order ever changes, existing license.dat files signed with the old order become unverifiable. Signature verification on client side would fail.

**Fix:** Use `serde_json::to_value` + sort keys explicitly, or use a BTreeMap:

```rust
pub fn to_canonical_json(payload: &LicensePayload) -> Result<Vec<u8>, String> {
    let value = serde_json::to_value(payload)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    // serde_json::Value serializes Map keys in insertion order,
    // but serde_json::Map is a BTreeMap by default (sorted)
    serde_json::to_vec(&value).map_err(|e| format!("JSON serialization failed: {}", e))
}
```

Note: `serde_json::Map` is backed by `BTreeMap` when the `preserve_order` feature is NOT enabled (which it isn't in this Cargo.toml). So `to_value` + `to_vec` gives true alphabetical key sorting.

---

## High Priority

### H1. PIN string lives on heap after spawn_blocking (commands/license_gen.rs:97-103)

The PIN is extracted as a `String` (`pin_str`) and then moved into `spawn_blocking`. After the closure completes, the String is dropped but not zeroized. The rest of the codebase uses `secrecy::Secret<String>` with `Zeroizing` for PIN storage.

Inside the closure (line 148), it is wrapped in `secrecy::Secret::new(pin_str)` but only at the point of use. Between extraction (line 97) and wrapping (line 148), the PIN lives as a plain String.

**Impact:** PIN may persist in freed heap memory. Low exploitability on desktop, but inconsistent with the zeroizing discipline used elsewhere.

**Fix:** Use `secrecy::SecretString` from extraction point:

```rust
let pin = secrecy::SecretString::new(
    login.get_pin().ok_or("PIN not available")?.to_string()
);
```

Then expose it via `pin.expose_secret()` at the C_Login call.

### H2. No input validation on `limit` / `offset` in list_license_audit (commands/license_gen.rs:212-215)

The command accepts any `i64` values. Negative values, extremely large limits, or negative offsets pass through to SQLite unchecked.

**Impact:** Negative offset/limit could cause unexpected query behavior. Extremely large limit could cause OOM.

**Fix:** Clamp values:

```rust
let limit = limit.clamp(1, 1000);
let offset = offset.max(0);
```

### H3. MachineCredential serde alias + rename_all interaction (license_gen/mod.rs:9-22)

`rename_all = "camelCase"` with `alias = "token_serial"` on a field named `token_serial`. The camelCase rename makes the primary deserialization key `tokenSerial`, and the alias adds `token_serial` as an alternative. For **serialization**, the output key will be `tokenSerial`.

However, when the frontend sends `MachineCredential` back to `generate_license`, it sends camelCase (`tokenSerial`). This works. But the `build_payload` function (payload.rs:28-44) copies `cred.token_serial` into `LicensePayload.token_serial` which serializes as `token_serial` (no rename_all on LicensePayload).

This means the **license payload JSON** uses snake_case keys (`token_serial`, `board_serial`, `cpu_id`, etc.) while the **Tauri command responses** use camelCase. This is likely intentional (payload is an internal format), but should be documented.

---

## Medium Priority

### M1. write_license_file overwrites without warning (signer.rs:40-57)

If a license file for the same user already exists, `std::fs::write` silently overwrites it. No backup, no confirmation.

**Impact:** Previously generated license for the same user is destroyed. Audit record points to same path but content changed.

**Fix:** Check existence first, or append timestamp to filename:

```rust
let file_name = format!("{}-{}-license.dat", safe_name, chrono::Utc::now().format("%Y%m%d%H%M%S"));
```

### M2. LicenseGenPage.tsx exceeds 130 lines with inline styles

The page has complex inline styles throughout. Follows the pattern of other pages in this codebase, but the 130+ lines with dense inline styles reduce maintainability.

### M3. No loading state for audit history (use-license-gen.ts:26-33)

`loadAuditHistory` silently catches all errors. If the table exists but the query is slow, the UI shows "No licenses generated yet" indefinitely with no loading indicator.

### M4. Credential file path not validated for existence before read (commands/license_gen.rs:54-56)

`import_credential` reads whatever path the frontend provides. The `tokio::fs::read_to_string` will fail with an IO error, which is handled, but the error message ("Cannot read file: ...") could leak filesystem structure.

**Fix:** Sanitize path info in error messages:

```rust
.map_err(|_| "Cannot read the selected credential file".to_string())?;
```

---

## Low Priority

### L1. Hardcoded product name (payload.rs:40)

`product: "CAHTQT_CLIENT".to_string()` is hardcoded. If multiple products need licensing in the future, this would need refactoring.

### L2. Unused `loadAuditHistory` in hook return (use-license-gen.ts:85)

`loadAuditHistory` is returned from the hook but never used by the consuming component. The audit history refresh after generation is handled internally.

---

## Positive Observations

1. **OperationGuard pattern** correctly prevents concurrent PKCS#11 sessions
2. **CKR_USER_ALREADY_LOGGED_IN handling** (line 153) is defensive and correct
3. **Filename sanitization** in `write_license_file` properly replaces non-alphanumeric chars
4. **Migration 005** uses `CREATE TABLE IF NOT EXISTS` and proper index
5. **Serde camelCase** alignment between Rust command responses and TS types is correct
6. **Manual mapping** from `LicenseAuditRow` to `LicenseAuditEntry` avoids leaking DB internals (output_path, cpu_id, board_serial excluded from frontend type)
7. **RAII session cleanup** -- PKCS#11 session + Pkcs11 instance dropped at end of spawn_blocking

## Recommended Actions (Priority Order)

1. **[C1]** Handle audit insert errors -- at minimum log, ideally surface as warning
2. **[C2]** Use explicit key sorting for canonical JSON instead of relying on struct field order
3. **[H1]** Wrap PIN in SecretString from extraction point
4. **[H2]** Clamp limit/offset inputs
5. **[M1]** Add timestamp or sequence to license filename to prevent overwrite

## Unresolved Questions

1. Is the license payload format (snake_case keys) documented in a spec shared with the client-side verification code? If verification expects different key casing, signatures will fail.
2. Is there a maximum credential file size that should be enforced to prevent DoS via large file reads?
3. Should `generate_license` verify that `expires_at` is in the future when not null?

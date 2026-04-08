# Brainstorm: License Gen Page (Server Side)

**Date**: 2026-04-07
**Branch**: feature/licenseGen
**Spec**: `feature/3. licenseGen/2F_Hardware_Bound_License_CAHTQT_Spec.docx` — Section 3

## Problem Statement

CAHTQT PKI Server app needs License Gen page — admin imports client Machine Credential JSON, signs license payload via server Bit4ID token (PKCS#11), outputs `license.dat`, tracks issuance history in SQLite.

## Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| UI approach | New page in Tauri app | Consistent with existing Encrypt/Decrypt/Partners/Settings pages |
| Token auth | Reuse existing token login | Leverage AppState.token_login, same PIN dialog flow |
| Credential import | File picker dialog | Standard pattern, parse + validate immediately |
| Expiry default | 1-year, configurable | Pre-fill 1yr, toggle perpetual option |
| Audit storage | SQLite + table view | More queryable, consistent with existing DB pattern |
| Output path | Fixed output directory | Same output_data_dir as encrypt. Predictable location |
| Metadata | Minimal auto-fill | product='CAHTQT_CLIENT', unit_name from settings. One-click sign |

## Recommended Approach: Lean Single-Command Flow

### Architecture

| Layer | Files | Responsibility |
|-------|-------|---------------|
| Frontend | `src/pages/LicenseGenPage.tsx` | Import button, credential preview, sign button, audit table |
| Tauri API | `src/lib/tauri-api.ts` (additions) | `importCredential`, `generateLicense`, `listLicenseAudit` |
| Commands | `src-tauri/src/commands/license_gen.rs` | Tauri command handlers |
| Core | `src-tauri/src/license_gen/mod.rs` | Orchestration |
| Core | `src-tauri/src/license_gen/payload.rs` | LicensePayload struct, canonical JSON |
| Core | `src-tauri/src/license_gen/signer.rs` | PKCS#11 signing, license.dat assembly |
| DB | `src-tauri/src/db/license_audit_repo.rs` | CRUD for license_audit table |

### Signing Flow (Rust)

1. Parse credential JSON → validate (no empty/placeholder values)
2. Compute `machine_fp = hex(SHA-256(cpu_id:board_serial)[0..8])`
3. Build LicensePayload (sorted keys, canonical UTF-8 JSON)
4. `digest = SHA-256(payload_bytes)`
5. Open PKCS#11 session (reuse token login state)
6. `C_Sign(RsaPkcs, priv_key, digest)` → 256-byte RSA signature
7. Assemble: `Base64(payload_bytes ‖ b"||SIG||" ‖ sig_bytes)` → write to output dir
8. Insert audit record into SQLite

### Page Layout

- Import Credential File button (file picker)
- Credential Preview card (user_name, token serial masked, cpu_id, board_serial, computed machine_fp)
- Expiry date picker + perpetual toggle
- Generate License button (disabled if no token logged in)
- Status message after generation
- License History table (from SQLite audit)

## Risks

- Token session conflicts with encrypt/decrypt — respect `is_operation_running` guard
- Key selection — use existing `CKA_SIGN=true` pattern
- Canonical JSON ordering — must match spec exactly for signature verification

## Next Steps

Create implementation plan with phased approach.

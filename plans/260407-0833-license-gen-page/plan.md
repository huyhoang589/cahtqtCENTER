---
title: "License Gen Page — Server-Side License Generation"
description: "Add License Gen page to CAHTQT PKI Server app for signing hardware-bound licenses via PKCS#11 token"
status: complete
priority: P1
effort: 6h
effort_actual: 5.5h
branch: feature/licenseGen
tags: [feature, backend, frontend, database, pkcs11, crypto]
blockedBy: []
blocks: []
created: 2026-04-07
completed: 2026-04-07
---

# License Gen Page — Server-Side License Generation

## Overview

Add a License Gen page to the CAHTQT PKI Server app. Admin imports client Machine Credential JSON, signs license payload using server Bit4ID token (PKCS#11 C_Sign), outputs `license.dat`, and tracks issuance history in SQLite.

**Spec reference:** `feature/3. licenseGen/2F_Hardware_Bound_License_CAHTQT_Spec.docx` — Section 3
**Brainstorm:** `plans/reports/brainstorm-260407-0833-license-gen-page.md`

## Cross-Plan Dependencies

None — both existing plans (`260405-*`) are complete.

## Phases

| Phase | Name | Status | Effort |
|-------|------|--------|--------|
| 1 | [Database Migration + Audit Repo](./phase-01-database-migration-and-audit-repo.md) | Complete | 0.5h |
| 2 | [Core License Gen Module](./phase-02-core-license-gen-module.md) | Complete | 2h |
| 3 | [Tauri Commands](./phase-03-tauri-commands.md) | Complete | 1h |
| 4 | [Frontend Page + Routing](./phase-04-frontend-page-and-routing.md) | Complete | 2h |
| 5 | [Integration + Compile Verify](./phase-05-integration-and-compile-verify.md) | Complete | 0.5h |

## Dependencies

- Phase 1 → Phase 2 → Phase 3 (sequential backend chain)
- Phase 3 → Phase 4 (frontend needs command API)
- Phase 5 depends on all prior phases
- New crates needed: `sha2`, `hex`, `base64`

## Validation Log

### Session 1 — 2026-04-07

#### Validated Decisions
1. **Signing mechanism**: RSA-PSS (`RsaPkcsPss`, SHA-256, salt=32) — proven with Bit4ID token in existing encrypt callbacks
2. **Session helper**: ~~Extract shared `open_pkcs11_session()` in `etoken/token_manager.rs`~~ **SIMPLIFIED**: generate_license opens PKCS#11 session directly. Avoids refactoring TokenContext, simpler approach.
3. **Token serial**: Read during session open via `get_token_info(slot)` — no dependency on prior scan cache
4. **Error UX**: Generic success/error messages — no CLI exit codes in GUI

#### Implementation Complete — 2026-04-07
- Phase 1: Database migration `005_license_audit.sql` + `license_audit_repo.rs`
- Phase 2: Core `license_gen/` module (MachineCredential, payload, signer)
- Phase 3: Tauri commands (import_credential, generate_license, list_license_audit)
- Phase 4: Frontend page + routing + sidebar integration
- Phase 5: cargo build + tsc --noEmit both pass cleanly

**Deviations from plan:**
- Did NOT extract shared `open_pkcs11_session()` helper. Phase 3 command opens session directly (simpler, no TokenContext refactoring needed)
- All compile checks passed, feature ready for merge

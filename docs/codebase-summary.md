# Codebase Summary

**Project:** CAHTQT — PKI Encryption Desktop App  
**Last Updated:** 2026-04-07  
**Status:** Feature-complete (License Gen v1.1 with export/delete/folder management)

## Overview

CAHTQT is a desktop application for batch M×N PKI encryption using PKCS#11 tokens. Users select M files and N recipients, encrypt in a single batch operation, and decrypt received files via their smart card or HSM token.

**Tech Stack:**
- **Frontend:** React 18 + TypeScript + Vite
- **Desktop:** Tauri v2 (Rust backend)
- **Crypto:** FFI bridge to `htqt_crypto.dll` (native C library)
- **Database:** SQLite (via sqlx)
- **Platform:** Windows 10/11 (native)

## Directory Structure

```
cahtqt-center/
├── src/                          # React frontend
│   ├── components/               # UI components (encrypt, decrypt, panels)
│   ├── pages/                    # Page-level components
│   ├── hooks/                    # Custom hooks (useEncrypt, useDecrypt)
│   ├── contexts/                 # Context providers (settings, token status)
│   ├── lib/                      # Tauri API bindings
│   ├── types/                    # TypeScript type definitions
│   ├── App.tsx                   # Root component
│   └── main.tsx                  # Entry point
│
├── src-tauri/src/                # Rust backend (Tauri commands)
│   ├── htqt_ffi/                 # FFI bridge to crypto DLL
│   │   ├── lib_loader.rs         # Dynamic library loader
│   │   ├── types.rs              # Rust FFI type bindings
│   │   ├── callbacks.rs          # Callback implementations
│   │   ├── token_context.rs      # PKCS#11 token session management
│   │   ├── error_codes.rs        # Error code definitions
│   │   └── mod.rs                # Module exports
│   │
│   ├── commands/                 # Tauri commands (RPC endpoints)
│   │   ├── encrypt.rs            # Batch encrypt command
│   │   ├── decrypt.rs            # Batch decrypt command
│   │   ├── etoken/               # Token management commands
│   │   ├── partners.rs           # Partner/recipient management
│   │   ├── settings.rs           # Settings persistence
│   │   ├── settings_cert.rs      # Certificate settings
│   │   ├── communication.rs      # App-level communication
│   │   ├── logs.rs               # Log retrieval
│   │   └── mod.rs                # Command registration
│   │
│   ├── db/                       # SQLite repositories
│   │   ├── partners_repo.rs      # Partner persistence
│   │   ├── partner_members_repo.rs # Recipient group members
│   │   ├── settings_repo.rs      # Settings storage
│   │   ├── logs_repo.rs          # Operation log storage
│   │   └── mod.rs                # DB initialization
│   │
│   ├── etoken/                   # PKCS#11 integration
│   │   ├── token_manager.rs      # Token enumeration & detection
│   │   ├── certificate_reader.rs # Certificate extraction
│   │   ├── certificate_exporter.rs # Export certificate chains
│   │   ├── library_detector.rs   # Auto-detect PKCS#11 libraries
│   │   ├── models.rs             # Token-related data structures
│   │   └── mod.rs                # Module exports
│   │
│   ├── cert_parser.rs            # X.509 certificate parsing
│   ├── app_log.rs                # Frontend logging bridge
│   ├── ffi_helpers.rs            # C string/buffer utilities
│   ├── lock_helper.rs            # Operation state locking
│   ├── output_dir.rs             # Output directory resolution
│   ├── models.rs                 # Shared data structures
│   ├── lib.rs                    # Tauri app initialization
│   └── main.rs                   # Entry point
│
├── feature/
│   └── 1. api-sf-type/           # New API header (v2)
│       └── htqt-api.h            # Crypto DLL API spec
│
├── plans/                        # Implementation plans
│   ├── 260405-0825-crypto-api-migration/
│   │   ├── plan.md               # Plan overview & validation
│   │   ├── phase-01-*.md         # FFI types & loader
│   │   ├── phase-02-*.md         # Encrypt command
│   │   └── phase-03-*.md         # Decrypt command
│   └── reports/                  # Brainstorm & research docs
│
├── docs/                         # Project documentation
│   ├── codebase-summary.md       # This file
│   ├── system-architecture.md    # Architecture & design
│   ├── code-standards.md         # Coding conventions
│   ├── project-changelog.md      # Release notes
│   └── development-roadmap.md    # Feature roadmap
│
├── src-tauri/migrations/         # SQLite schema migrations
├── src-tauri/capabilities/       # Tauri security ACLs
├── src-tauri/icons/              # App icons
├── package.json                  # Node.js dependencies
├── Cargo.toml                    # Rust dependencies
├── tsconfig.json                 # TypeScript config
├── vite.config.ts                # Vite build config
├── tauri.conf.json               # Tauri config
└── README.md                     # Quick-start guide
```

## Key Components

### Frontend (React/TypeScript)

**Pages:**
- **EncryptPage** — File/recipient selection, batch encrypt workflow
- **DecryptPage** — File selection, batch decrypt workflow
- **PartnersPage** — Recipient/group management
- **SettingsPage** — Token, library path, output directory config

**Hooks:**
- `useEncrypt` — Manage encrypt command state & progress
- `useDecrypt` — Manage decrypt command state & progress
- `useTokenStatus` — Token login state subscription
- `useSettingsStore` — Persistent settings state
- `useFileStatuses` — Per-file operation progress tracking

**Components:**
- `EncryptProgressPanel` — Real-time encrypt progress display
- `DecryptProgressPanel` — Real-time decrypt progress display
- `RecipientTable` — Select recipients for encryption
- `CertificateTable` — View token certificates
- `LoginTokenModal` — PIN entry for token login

### Backend (Rust/Tauri)

**Crypto FFI (`htqt_ffi/`):**
- Dynamically loads `htqt_crypto.dll` at runtime
- Exposes wrapper methods: `enc_multi()`, `dec_sf()`
- Manages DLL lock for thread-safe calls
- Converts Rust types ↔ C FFI structs

**Commands:**
- `encrypt_batch()` — Batch encrypt M files × N recipients
- `decrypt_batch()` — Batch decrypt M `.sf1` files
- `import_credential(path)` — Parse + validate Machine Credential JSON
- `generate_license(credential, expires_at, unit_name)` — Sign license via PKCS#11 token
- `list_license_audit(limit, offset)` — Paginated license generation history
- `export_license(audit_id)` — Export stored license blob to disk
- `delete_license(audit_id)` — Hard-delete audit record and file
- `open_license_folder(user_name)` — Open license directory in file explorer
- `scan_tokens()` — Enumerate PKCS#11 tokens
- `get_token_certificates()` — List certificates on token
- `login_token()` — Unlock token with PIN
- `logout_token()` — Lock token session

**PKCS#11 Integration:**
- `TokenManager` — Enumerate available tokens
- `CertificateReader` — Extract certificates from token
- Auto-detect PKCS#11 library paths
- Session lifecycle management (open/close)

**Database:**
- SQLite schema: partners, partner_members, settings, logs
- Migrations tracked in `migrations/`
- Repository pattern for data access

## Crypto API Specification (v2)

**File:** `feature/1. api-sf-type/htqt-api.h`

### Encryption API

```c
int encHTQT_sf_multi(
    const BatchEncryptParams *params,
    const CryptoCallbacksV2  *cbs,
    BatchResult              *results,
    char                     *error_msg,
    int                       error_len);
```

**Inputs:**
- `params` → File list, recipient list, output dir, flags
- `cbs` → Crypto callbacks (sign, RSA-OAEP encrypt, progress)
- `results` → Array sized to `file_count` (not `file_count × recipient_count`)

**Outputs:**
- One `.sf1` file per input file (all N recipients embedded)
- `results[i].output_path` — Path to encrypted file
- `results[i].status` — HTQT_OK or HTQT_ERR_*

### Decryption API

```c
int decHTQT_sf(
    const BatchSfDecryptParams *params,
    const CryptoCallbacksV2    *cbs,
    BatchResult                *results,
    char                       *error_msg,
    int                         error_len);
```

**Inputs:**
- `params` → SF file list, output dir, flags
- `cbs` → Crypto callbacks (decrypt, RSA-OAEP, verify signature)

**Outputs:**
- One decrypted file per `.sf1` input
- Output filename from SF header `orig_name` field
- `results[i].output_path` — Full path to decrypted file

## Data Flow

### Encrypt Workflow

```
User selects files + recipients
         ↓
encrypt_batch() command
         ↓
Build BatchEncryptParams (FileEntry[], RecipientEntry[])
         ↓
Open PKCS#11 token session
         ↓
Call encHTQT_sf_multi() via FFI
    ├─ sign_fn callback (PKCS#11)
    ├─ rsa_enc_cert_fn callback (public key from certs)
    └─ progress_fn callback (UI updates)
         ↓
M × N encryption pairs → M × 1 .sf1 files
         ↓
Write to output_dir/{file_id}.sf1
         ↓
Close token session
         ↓
Return EncryptResult (success_count, errors)
```

### Decrypt Workflow

```
User selects .sf1 files
         ↓
decrypt_batch() command
         ↓
Build BatchSfDecryptParams (FileEntry[], output_dir)
         ↓
Open PKCS#11 token session
         ↓
Call decHTQT_sf() via FFI
    ├─ rsa_dec_fn callback (PKCS#11 decrypt)
    ├─ verify_fn callback (signature verification)
    └─ progress_fn callback (UI updates)
         ↓
Read BatchResult[].output_path from DLL
         ↓
Files written to output_dir/{orig_name}
         ↓
Close token session
         ↓
Return DecryptResult with paths
```

## Configuration & Runtime

**Settings (SQLite):**
- Crypto DLL path (default: `crypto_dll.dll` in app directory)
- PKCS#11 library path (auto-detected or manual)
- Output directory override
- Last token scan results

**Environment:**
- Windows 10/11 only
- PKCS#11 library (eToken, Thales, etc.)
- `crypto_dll.dll` (v2 with SF v1 format support)

## Recent Changes

**2026-04-07: License Change v1 — Export, Delete, and Folder Management**

- **Database:** Migration `006_license_audit_blob.sql` adds `license_blob TEXT` to `license_audit` table
- **License Gen Commands:** Added 3 new commands:
  - `export_license(audit_id)` — Retrieve + write stored license blob
  - `delete_license(audit_id)` — Hard-delete audit record + disk file
  - `open_license_folder(user_name)` — Open license dir in file explorer
- **Utilities:** Added `sanitize_user_name()` helper for safe filesystem paths
- **Output Path:** Updated to `{base}\SF\LICENSE\{sanitized_user_name}\{sanitized_user_name}-license.dat`
- **Security:** Path traversal protection in delete/export via expected path reconstruction

**2026-04-05: Crypto API Migration (SF v1 Format)**

- **Encrypt API:** `encHTQT_multi` → `encHTQT_sf_multi`
  - Results array now sized to `file_count` (was `file_count × recipient_count`)
  - One `.sf1` file per input (all N recipients embedded)
  - Progress callback still per-(file, recipient) pair

- **Decrypt API:** `decHTQT_v2` → `decHTQT_sf` (batch params struct)
  - Accepts `BatchSfDecryptParams` (file list + output dir)
  - Output filenames taken from SF header `orig_name`
  - Batch result array sized to `file_count`

- **FFI Types:**
  - Added `FnEncHTQTSfMulti`, `FnDecHTQTSf` function pointer types
  - Added `BatchSfDecryptParams` struct
  - Updated `HtqtLib` field names: `enc_sf_multi_fn`, `dec_sf_fn`

- **Rust Commands:**
  - `encrypt.rs`: Results vec capacity updated, per-file iteration
  - `decrypt.rs`: Single `dec_sf()` batch call, output path from `BatchResult`

---

**See Also:**
- System Architecture: `docs/system-architecture.md`
- Code Standards: `docs/code-standards.md`
- Changelog: `docs/project-changelog.md`
- Roadmap: `docs/development-roadmap.md`

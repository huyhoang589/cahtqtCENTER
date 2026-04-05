# System Architecture

**Last Updated:** 2026-04-05  
**Status:** Updated for Crypto API v2 (SF v1 format migration)

## High-Level Architecture

CAHTQT is a desktop application with three main layers:

```
┌─────────────────────────────────────────────────────────────┐
│                    React Frontend (TypeScript)              │
│  Encrypt/Decrypt Pages • Recipient Groups • Token Settings  │
│                   (Vite build + HMR dev)                    │
└────────────────────┬────────────────────────────────────────┘
                     │ Tauri IPC (JSON serialization)
                     ↓
┌─────────────────────────────────────────────────────────────┐
│              Tauri Backend (Rust) — Commands                │
│  ├─ encrypt_batch()     ├─ scan_tokens()                    │
│  ├─ decrypt_batch()     ├─ get_token_certificates()        │
│  ├─ login_token()       └─ Settings persistence            │
│                                                              │
│  Supporting Modules:                                         │
│  ├─ PKCS#11 Integration (etoken/)                          │
│  ├─ Crypto FFI (htqt_ffi/)                                 │
│  ├─ Database (db/)                                          │
│  └─ Certificate Parsing (cert_parser.rs)                    │
└─────────────────────┬──────────────────────────────────────┘
                      │ FFI calls
                      ↓
┌─────────────────────────────────────────────────────────────┐
│     htqt_crypto.dll (C) — SF v1 Batch Encryption            │
│  ├─ encHTQT_sf_multi()   ├─ RSA-PSS signing                │
│  ├─ decHTQT_sf()         ├─ RSA-OAEP encryption            │
│  └─ HTQT_GetError()      └─ Signature verification         │
└─────────────────────┬──────────────────────────────────────┘
                      │ PKCS#11 callbacks
                      ↓
┌─────────────────────────────────────────────────────────────┐
│           PKCS#11 Smart Card / HSM Token                    │
│  (eToken, Thales, etc.) — Holds signing key                │
└─────────────────────────────────────────────────────────────┘
```

## Layer Breakdown

### 1. Frontend (React/TypeScript)

**Responsibilities:**
- User interface for file/recipient selection
- Real-time operation progress display
- Settings configuration UI
- Token login/logout UI

**Key Components:**
- `EncryptPage` — M files + N recipients → encrypt workflow
- `DecryptPage` — Select `.sf1` files → decrypt workflow
- `PartnersPage` — Manage recipient groups and certificates
- `SettingsPage` — Configure token library, DLL path, output directory

**State Management:**
- `useSettingsStore` — Persistent settings (SQLite-backed)
- `useTokenStatus` — Token login state (session-scoped)
- `useEncrypt` / `useDecrypt` — Operation progress (temporary)

**Communication:**
- Tauri IPC invokes backend commands (JSON serialized)
- Listens to progress events via `appWindow.listen()`
- No direct access to file system or crypto operations

### 2. Backend (Rust/Tauri)

#### Command Handler Layer (`commands/`)

**`encrypt_batch()`**
1. Validate token logged-in status
2. Load user's certificate DER (from last token scan)
3. Build `BatchEncryptParams`:
   - `files[]` — One `FileEntry` per selected input file
   - `recipients[]` — One `RecipientEntry` per recipient certificate
   - `output_dir` — From settings or override
4. Call `HtqtLib::enc_multi()` (FFI wrapper)
5. Emit progress events for each (file, recipient) pair
6. Return `EncryptResult` with success count and errors

**`decrypt_batch()`**
1. Validate token logged-in status
2. Load user's certificate DER (for fingerprint matching)
3. Build `BatchSfDecryptParams`:
   - `files[]` — One `FileEntry` per selected `.sf1` file
   - `output_dir` — From settings
4. Call `HtqtLib::dec_sf()` (FFI wrapper)
5. Emit progress events as files decrypt
6. Return decrypted file paths from `BatchResult.output_path`

**Token Management:**
- `login_token()` — Open PKCS#11 session, cache PIN in memory
- `logout_token()` — Close session, clear PIN
- `scan_tokens()` — Enumerate tokens via PKCS#11 library

#### PKCS#11 Integration (`etoken/`)

**`TokenManager`**
- Enumerates available token slots
- Detects token type (eToken, Thales, etc.)
- Auto-discovers PKCS#11 library path

**`CertificateReader`**
- Opens token session with PIN
- Extracts certificates from private key objects
- Builds certificate chain

**`LibraryDetector`**
- Searches Windows registry for PKCS#11 DLLs
- Validates library by attempting symbol resolution
- Fallback to user-configured path

#### Crypto FFI Layer (`htqt_ffi/`)

**`HtqtLib`**
- Dynamically loads `htqt_crypto.dll` at runtime
- Manages DLL symbol resolution
- Provides thread-safe wrapper methods

**Key Methods:**
```rust
pub fn enc_multi(
    &self,
    params: &BatchEncryptParams,
    cbs: &CryptoCallbacksV2,
    results: &mut [BatchResult],
) -> Result<i32, String>

pub fn dec_sf(
    &self,
    params: &BatchSfDecryptParams,
    cbs: &CryptoCallbacksV2,
    results: &mut [BatchResult],
) -> Result<i32, String>
```

**Callback Implementation (`callbacks.rs`):**
- `sign_fn` — Call PKCS#11 token to sign digest
- `rsa_enc_cert_fn` — Use public key from cert to encrypt
- `rsa_dec_fn` — Call PKCS#11 token to decrypt
- `verify_fn` — Use sender's cert to verify signature

#### Database Layer (`db/`)

**Repositories (SQLite):**
- `SettingsRepo` — Persist user config (DLL path, output dir, etc.)
- `PartnersRepo` — Recipient groups
- `PartnerMembersRepo` — Recipients within groups
- `LogsRepo` — Operation history (encryption/decryption audit trail)

**Schema:**
```sql
-- Settings (key-value store)
settings (key TEXT PRIMARY KEY, value TEXT)

-- Recipient groups
partners (id INTEGER PRIMARY KEY, name TEXT, created_at DATETIME)

-- Group members (individual recipients)
partner_members (id INTEGER, partner_id INTEGER, cert_der BLOB, 
                 commonName TEXT, ...)

-- Operation log
logs (id INTEGER PRIMARY KEY, operation TEXT, success INTEGER, 
      file_count INTEGER, created_at DATETIME)
```

### 3. Crypto DLL (Native C Library)

**API v2 (htqt-api.h):**

```c
/* Batch encrypt M files × N recipients → M SF v1 files */
int encHTQT_sf_multi(
    const BatchEncryptParams *params,
    const CryptoCallbacksV2  *cbs,
    BatchResult              *results,
    char                     *error_msg,
    int                       error_len);

/* Batch decrypt M SF v1 files */
int decHTQT_sf(
    const BatchSfDecryptParams *params,
    const CryptoCallbacksV2    *cbs,
    BatchResult                *results,
    char                       *error_msg,
    int                         error_len);
```

**Input Structures:**

`BatchEncryptParams`:
- `files[]` — Array of file paths to encrypt
- `recipients[]` — Array of recipient certificates
- `output_dir` — Output directory
- `flags` — HTQT_BATCH_CONTINUE_ON_ERROR, HTQT_BATCH_OVERWRITE_OUTPUT

`BatchSfDecryptParams`:
- `files[]` — Array of `.sf1` file paths
- `output_dir` — Decrypt output directory
- `flags` — Same flags as encryption

**Output Structure:**

`BatchResult` (array with one entry per file):
- `file_index` — Index into input files array
- `recipient_index` — (Encrypt only) recipient index
- `status` — HTQT_OK or error code
- `output_path[512]` — Path to generated file
- `error_detail[256]` — Error message (if status != HTQT_OK)

**SF v1 File Format:**
- Header: magic number, version, orig_name (original filename)
- RecipientBlocks: one per recipient, encrypted session key + signature
- Payload: AES-encrypted file data

## Data Flow Diagrams

### Encryption Flow

```
User clicks "Encrypt"
    │
    ├─ Select M files (EncryptPage)
    ├─ Select N recipients (RecipientTable)
    ├─ Confirm operation (ConfirmEncryptDialog)
    │
    └─→ encrypt_batch(src_paths, partner_name, cert_paths)
        │
        ├─ Check token login status (PKCS#11 session)
        ├─ Load own_cert_der (from last token scan)
        │
        └─→ Build BatchEncryptParams
            │
            ├─ FileEntry[]: one per src file
            ├─ RecipientEntry[]: certs from partner group
            ├─ output_dir: settings.output_dir
            │
            └─→ HtqtLib::enc_multi() (FFI call)
                │
                ├─→ encHTQT_sf_multi() in DLL
                │   │
                │   ├─ For each (file, recipient) pair:
                │   │   ├─ sign_fn() → PKCS#11 token signs digest
                │   │   ├─ rsa_enc_cert_fn() → Encrypt session key
                │   │   └─ progress_fn() → UI updates
                │   │
                │   └─ Write {file_id}.sf1
                │
                ├─ results[file_idx].output_path
                │
                └─→ Return EncryptResult
                    │
                    └─ UI: Show success, file paths, errors
```

### Decryption Flow

```
User clicks "Decrypt"
    │
    ├─ Select M .sf1 files (DecryptPage)
    ├─ Confirm operation
    │
    └─→ decrypt_batch(sf1_paths)
        │
        ├─ Check token login status
        ├─ Load own_cert_der (for fingerprint matching)
        │
        └─→ Build BatchSfDecryptParams
            │
            ├─ FileEntry[]: one per .sf1 file
            ├─ output_dir: settings.output_dir
            │
            └─→ HtqtLib::dec_sf() (FFI call)
                │
                ├─→ decHTQT_sf() in DLL
                │   │
                │   ├─ For each .sf1 file:
                │   │   ├─ rsa_dec_fn() → PKCS#11 token decrypts key
                │   │   ├─ verify_fn() → Verify sender signature
                │   │   └─ progress_fn() → UI updates
                │   │
                │   └─ Read orig_name from SF header
                │       Write to output_dir/{orig_name}
                │
                ├─ results[file_idx].output_path (from DLL)
                │
                └─→ Return DecryptResult
                    │
                    └─ UI: Show success, file paths, errors
```

## Thread Safety & Concurrency

**Operation Lock:**
- `OperationGuard` — Prevents concurrent encrypt/decrypt operations
- One operation at a time (token session is single-threaded)

**DLL Lock:**
- `DLL_LOCK` (Mutex) — Serializes all FFI calls to crypto DLL
- Some DLLs (especially with PKCS#11) are not thread-safe

**Token Session:**
- PKCS#11 sessions are typically single-threaded
- Session ID cached in `TokenLoginState` during login
- All crypto callbacks run within single operation

## Key Architectural Decisions

### 1. Batch APIs Over Per-File APIs

**Decision:** Use `encHTQT_sf_multi()` (M files → M results) instead of per-file encryption.

**Rationale:**
- Amortize overhead of token session setup
- Single progress callback for all pairs
- Results array properly sized (not M×N)

### 2. Output Filename from SF Header

**Decision:** Decrypt writes `{orig_name}` from SF header, not derived from input `.sf1` filename.

**Rationale:**
- Preserves original filename intent
- Sender controls output filename (not recipient)
- Supports anonymous `.sf1` transfers

### 3. FFI-Level Callbacks vs Higher-Level Abstraction

**Decision:** Implement callbacks at FFI layer, not higher.

**Rationale:**
- DLL directly calls into PKCS#11 token (no context switching)
- Performance critical for large files
- Matches crypto DLL design intent

### 4. SQLite for Persistence

**Decision:** Use SQLite for settings, logs, and recipient management.

**Rationale:**
- Lightweight, no external database required
- Built-in with sqlx
- Suitable for single-user desktop app

### 5. Tauri v2 Desktop Framework

**Decision:** Use Tauri v2 over Electron.

**Rationale:**
- Smaller bundle size
- Memory-efficient
- Rust backend natural fit for FFI integration

## API Contracts

### Encrypt Command

**Input:**
```typescript
{
  src_paths: string[],      // Absolute paths to files
  partner_name: string,     // Recipient group name
  cert_paths: string[],     // Absolute paths to .pem/.der certs
  output_dir?: string       // Override output directory
}
```

**Output:**
```typescript
{
  total: number,
  success_count: number,
  error_count: number,
  errors: string[]
}
```

**Progress Events:**
```typescript
// Emitted per (file, recipient) pair
{
  current: number,
  total: number,
  file_name: string,
  file_path: string,
  status: "processing" | "success" | "warning" | "error",
  error?: string
}
```

### Decrypt Command

**Input:**
```typescript
{
  sf1_paths: string[],      // Absolute paths to .sf1 files
  output_dir?: string       // Override output directory
}
```

**Output:**
```typescript
{
  total: number,
  success_count: number,
  error_count: number,
  errors: string[],
  output_paths: string[]    // Decrypted file paths
}
```

## Error Handling

**Error Codes (from `htqt_ffi/error_codes.rs`):**
- `HTQT_OK` (0) — Success
- `HTQT_ERR_INVALID_PARAMS` — Malformed input
- `HTQT_ERR_FILE_NOT_FOUND` — Input file missing
- `HTQT_ERR_CERT_INVALID` — Invalid certificate
- `HTQT_ERR_CRYPTO_FAILED` — Cryptographic operation failed
- `HTQT_ERR_PARTIAL` — Some files succeeded, some failed

**Error Handling Strategy:**
1. DLL fills `BatchResult[i].error_detail` for failed items
2. Rust command aggregates errors into `EncryptResult.errors`
3. UI displays per-file errors in progress panel
4. Operation continues (`HTQT_BATCH_CONTINUE_ON_ERROR` flag)

## Configuration

**Settings (Persisted in SQLite):**
- `crypto_dll_path` — Path to htqt_crypto.dll (default: app directory)
- `pkcs11_lib_path` — PKCS#11 library path (auto-detected or manual)
- `output_dir` — Default output directory (user's Documents or app dir)

**Runtime Overrides:**
- `output_dir` optional parameter in encrypt/decrypt commands
- Can be overridden per-operation

## Performance Considerations

**Batch Processing:**
- M×N file pairs encrypted in single DLL call
- Single token session for all operations
- Results array pre-allocated

**Progress Updates:**
- Progress callback fired after each (file, recipient) pair
- Limited to ~1000 pairs for responsive UI (configurable)
- Large batches may need splitting

**Token Session Caching:**
- Session ID held in memory during operation
- Reused across encrypt/decrypt operations
- Cleared on logout

---

**See Also:**
- Codebase Summary: `docs/codebase-summary.md`
- Code Standards: `docs/code-standards.md`
- Changelog: `docs/project-changelog.md`

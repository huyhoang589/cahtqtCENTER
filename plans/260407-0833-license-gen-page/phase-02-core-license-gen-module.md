# Phase 2 — Core License Gen Module

## Context Links
- Spec: `feature/3. licenseGen/2F_Hardware_Bound_License_CAHTQT_Spec.docx` §3.1–3.3
- Existing PKCS#11 pattern: `src-tauri/src/htqt_ffi/token_context.rs`
- Token manager: `src-tauri/src/etoken/token_manager.rs`
- Brainstorm: `plans/reports/brainstorm-260407-0833-license-gen-page.md`

## Overview
- **Priority:** P1 (core business logic)
- **Status:** Complete
- **Description:** Implement Rust module for license payload construction, machine fingerprint computation, PKCS#11 signing, and license.dat assembly.

## Key Insights
- Signing uses existing PKCS#11 infra — `open_token_session()` pattern from `token_context.rs`
- License Gen does NOT use htqt_crypto.dll — it directly signs via PKCS#11 C_Sign
- Canonical JSON: keys sorted alphabetically, no trailing whitespace
- machine_fp = first 8 bytes of SHA-256 = 16 hex chars
- License binary: `payload_bytes || b"||SIG||" || sig_bytes`, then Base64-encoded
- RSA signature via `Mechanism::RsaPkcsPss` (RSA-PSS with SHA-256, salt=32) — proven with Bit4ID token in encrypt callbacks

## Requirements

### Functional
- Parse + validate Machine Credential JSON (token_serial, cpu_id, board_serial, user_name, registered_at)
- Compute machine_fp from cpu_id + board_serial
- Build LicensePayload with all required fields
- Sign payload digest via server token (PKCS#11 C_Sign)
- Assemble license.dat binary and Base64-encode
- Write to output directory

### Non-functional
- Signing must complete in <2s on typical hardware
- Validate credential fields: reject empty, placeholder ("To be filled by O.E.M."), or too-short values

## Architecture

```
license_gen/
├── mod.rs          — Public API: generate_license(), MachineCredential struct
├── payload.rs      — LicensePayload struct, canonical JSON serialization, machine_fp
└── signer.rs       — PKCS#11 signing, license.dat binary assembly
```

### Data Flow
```
credential.json → parse → validate → compute machine_fp
                                    → build LicensePayload
                                    → canonical JSON bytes
                                    → SHA-256 digest
                                    → C_Sign(digest) via token
                                    → assemble payload||SIG||signature
                                    → Base64 encode
                                    → write license.dat
```

## Related Code Files

### Create
- `src-tauri/src/license_gen/mod.rs`
- `src-tauri/src/license_gen/payload.rs`
- `src-tauri/src/license_gen/signer.rs`

### Modify
- `src-tauri/src/lib.rs` — add `pub mod license_gen;`
- `src-tauri/Cargo.toml` — add `sha2`, `hex`, `base64` dependencies

## Implementation Steps

### Step 1: Add dependencies to Cargo.toml
```toml
sha2 = "0.10"
hex = "0.4"
base64 = "0.22"
```

### Step 2: Create `src-tauri/src/license_gen/mod.rs`
```rust
pub mod payload;
pub mod signer;

use serde::{Deserialize, Serialize};

/// Machine Credential JSON from client Registration Tool
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MachineCredential {
    pub token_serial: String,
    pub cpu_id: String,
    pub board_serial: String,
    pub user_name: String,
    pub registered_at: String,
}

/// Validate credential fields — reject empty/placeholder values
pub fn validate_credential(cred: &MachineCredential) -> Result<(), String> {
    let checks = [
        ("token_serial", &cred.token_serial),
        ("cpu_id", &cred.cpu_id),
        ("board_serial", &cred.board_serial),
        ("user_name", &cred.user_name),
    ];
    for (field, value) in &checks {
        let v = value.trim();
        if v.is_empty() || v.len() < 3 {
            return Err(format!("Invalid {}: value too short or empty", field));
        }
        if v == "To be filled by O.E.M." || v == "Default string" || v == "UNAVAILABLE" {
            return Err(format!("Invalid {}: placeholder value '{}'", field, v));
        }
    }
    Ok(())
}
```

### Step 3: Create `src-tauri/src/license_gen/payload.rs`

Key elements:
- `LicensePayload` struct with `#[serde(serialize_with)]` for sorted keys
- `compute_machine_fp(cpu_id, board_serial) -> String` — SHA-256, first 8 bytes, hex
- `build_payload(cred, expires_at, server_serial) -> LicensePayload`
- `to_canonical_json(payload) -> Result<Vec<u8>, String>` — sorted keys JSON bytes

```rust
use sha2::{Sha256, Digest};
use serde::Serialize;

/// Compute machine fingerprint: hex(SHA-256(cpu_id:board_serial)[0..8])
pub fn compute_machine_fp(cpu_id: &str, board_serial: &str) -> String {
    let raw = format!("{}:{}", cpu_id, board_serial);
    let hash = Sha256::digest(raw.as_bytes());
    hex::encode(&hash[..8]) // 16 lowercase hex chars
}

#[derive(Debug, Serialize, Clone)]
pub struct LicensePayload {
    pub board_serial: String,
    pub cpu_id: String,
    pub expires_at: Option<i64>,   // null for perpetual
    pub issued_at: i64,
    pub issued_by: String,         // server token serial
    pub machine_fp: String,
    pub product: String,
    pub token_serial: String,
}
```

NOTE: struct fields in alphabetical order = serde serializes sorted keys by default.

### Step 4: Create `src-tauri/src/license_gen/signer.rs`

Key elements:
- `sign_payload(session, priv_key, payload_bytes) -> Result<Vec<u8>, String>` — SHA-256 digest → C_Sign
- `assemble_license_dat(payload_bytes, signature) -> String` — concat with separator, Base64
- `write_license_file(output_dir, user_name, content) -> Result<String, String>`

```rust
use sha2::{Sha256, Digest};
use cryptoki::mechanism::Mechanism;
use cryptoki::object::ObjectHandle;
use cryptoki::session::Session;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

const SEPARATOR: &[u8] = b"||SIG||";

/// Sign payload bytes: SHA-256 digest → C_Sign(RsaPkcsPss, salt=32)
pub fn sign_payload(
    session: &Session,
    priv_key: ObjectHandle,
    payload_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    let digest = Sha256::digest(payload_bytes);
    let mechanism = Mechanism::RsaPkcsPss(cryptoki::mechanism::rsa::PkcsPssParams::new(
        cryptoki::mechanism::MechanismType::SHA256,
        cryptoki::mechanism::rsa::PkcsMgfType::MGF1_SHA256,
        32, // salt length — matches existing encrypt callback pattern
    ));
    session
        .sign(&mechanism, priv_key, &digest)
        .map_err(|e| format!("PKCS#11 C_Sign failed: {}", e))
}

/// Assemble license.dat content: Base64(payload || SEPARATOR || signature)
pub fn assemble_license_dat(payload_bytes: &[u8], signature: &[u8]) -> String {
    let mut binary = Vec::with_capacity(payload_bytes.len() + SEPARATOR.len() + signature.len());
    binary.extend_from_slice(payload_bytes);
    binary.extend_from_slice(SEPARATOR);
    binary.extend_from_slice(signature);
    STANDARD.encode(&binary)
}
```

### Step 5: Add `pub mod license_gen;` to `src-tauri/src/lib.rs`

### Step 6: Run `cargo build` to verify compilation

## Todo List
- [x] Add `sha2`, `hex`, `base64` to Cargo.toml
- [x] Create `license_gen/mod.rs` with MachineCredential + validate_credential
- [x] Create `license_gen/payload.rs` with LicensePayload + compute_machine_fp + canonical JSON
- [x] Create `license_gen/signer.rs` with sign_payload + assemble_license_dat + write file
- [x] Add `pub mod license_gen;` to lib.rs
- [x] Run `cargo build` to verify compilation

## Success Criteria
- `cargo build` clean
- `compute_machine_fp("BFEBFBFF000906EA", "SN20240815XYZ01")` returns `"a3b1c9d4e8f20711"` (matches spec example — verify!)
- Canonical JSON has alphabetically sorted keys
- Sign function correctly uses `Mechanism::RsaPkcs` with SHA-256 digest

## Risk Assessment
- **LOW** (validated): Using `Mechanism::RsaPkcsPss` (SHA-256, salt=32) — proven with Bit4ID token in existing encrypt callbacks.
- **LOW**: Canonical JSON ordering — struct fields alphabetical = serde default sorted. Verify with test.
- **LOW**: machine_fp computation must exactly match spec. First 8 bytes = first 16 hex chars of SHA-256.

## Security Considerations
- Private key never leaves token hardware (C_Sign only)
- PIN not stored in license_gen module — obtained from AppState.token_login
- Credential file may contain PII (user_name) — no special handling needed on server side

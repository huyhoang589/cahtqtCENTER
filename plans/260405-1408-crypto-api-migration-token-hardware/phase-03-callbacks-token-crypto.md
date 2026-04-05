# Phase 3 — Callbacks: C_CreateObject Token Crypto

**Context:** `plans/reports/brainstorm-260405-1408-crypto-api-migration.md` | `htqt_ffi/callbacks.rs`
**Depends on:** Phase 1+2 complete

## Overview
- **Priority:** P1
- **Status:** Complete
- **Effort:** 1.5h
- Rewrite `cb_rsa_oaep_enc_cert` and `cb_rsa_pss_verify` to use PKCS#11 token hardware via `C_CreateObject` + `C_Encrypt`/`C_Verify`. Remove software RSA path for these two callbacks.

## Key Insights

### cb_rsa_oaep_enc_cert (currently software)
- Extracts recipient's public key from `cert_der` → encrypts plaintext via `rsa` crate
- New flow: extract RSA modulus + exponent bytes → `session.create_object()` with pub key attrs → `session.encrypt(CKM_RSA_PKCS_OAEP)` → `session.destroy_object()`
- `_user_ctx` → `user_ctx` (must cast to `*const TokenContext` — never null during encrypt)
- The `rsa` crate is still needed to extract modulus/exponent `BigUint` → bytes from DER

### cb_rsa_pss_verify (currently software with 2-salt retry)
- Tries salt=32 then max_salt to handle tokens that override salt on sign
- New flow: extract RSA pub key from sender cert → `C_CreateObject` → `session.verify(CKM_RSA_PKCS_PSS, salt=32)` → `C_DestroyObject`
- **Salt fixed at 32** — accepted risk: files signed with max-salt PSS will fail verify
- `_user_ctx` → `user_ctx`

### C_CreateObject attributes for RSA public key
```rust
use cryptoki::object::{Attribute, ObjectClass, KeyType};

let attrs = vec![
    Attribute::Class(ObjectClass::PUBLIC_KEY),
    Attribute::KeyType(KeyType::RSA),
    Attribute::Modulus(modulus_bytes),        // big-endian Vec<u8>
    Attribute::PublicExponent(exponent_bytes), // big-endian Vec<u8>
    Attribute::Encrypt(true),  // for enc callback
    Attribute::Verify(true),   // for verify callback
    Attribute::Token(false),   // session object, not persistent
    Attribute::Private(false),
];
let handle = ctx.session().create_object(&attrs)?;
// ... use handle ...
ctx.session().destroy_object(handle)?; // always cleanup
```

### Extracting modulus/exponent from cert DER
```rust
// Already have: extract_spki_der(cert_slice) -> Vec<u8>
// Then:
let pub_key = RsaPublicKey::from_public_key_der(&spki_der)?;
let modulus_bytes = pub_key.n().to_bytes_be();
let exponent_bytes = pub_key.e().to_bytes_be();
```
`to_bytes_be()` from `num_bigint::BigUint` — available via `rsa` crate's re-exports or `pub_key.n()` returns `&rsa::BigUint`.

Note: `rsa::BigUint` is `num_bigint::BigUint`. Use `.to_bytes_be()`.

## Related Code Files
- **Modify:** `src-tauri/src/htqt_ffi/callbacks.rs`

## Implementation Steps

### Step 1: Update imports
Remove unused imports after rewrite:
```rust
// Remove:
use rand::thread_rng;
use rsa::pss::{Signature as PssSignature, VerifyingKey};
use rsa::signature::hazmat::PrehashVerifier;
use rsa::traits::PublicKeyParts;
use rsa::{Oaep, RsaPublicKey};

// Keep (still needed for key extraction):
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPublicKey;

// Add:
use cryptoki::object::{Attribute, ObjectClass, KeyType};
```

### Step 2: Rewrite `cb_rsa_oaep_enc_cert`
Replace entire function body:
```rust
pub unsafe extern "C" fn cb_rsa_oaep_enc_cert(
    plaintext: *const u8, plaintext_len: u32,
    cert_der: *const u8, cert_der_len: u32,
    ciphertext_out: *mut u8, ciphertext_len: *mut u32,
    user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if plaintext.is_null() || cert_der.is_null() || ciphertext_out.is_null()
            || ciphertext_len.is_null() || user_ctx.is_null() {
            return -1;
        }
        let ctx = &*(user_ctx as *const TokenContext);
        let pt_slice = slice::from_raw_parts(plaintext, plaintext_len as usize);
        let cert_slice = slice::from_raw_parts(cert_der, cert_der_len as usize);

        // Extract RSA public key components from recipient cert
        let (modulus, exponent) = match extract_rsa_key_components(cert_slice) {
            Ok(kc) => kc,
            Err(e) => { eprintln!("[cb_enc_cert] key extract: {}", e); return -1; }
        };

        // Import recipient public key as session object on token
        let attrs = vec![
            Attribute::Class(ObjectClass::PUBLIC_KEY),
            Attribute::KeyType(KeyType::Rsa),
            Attribute::Modulus(modulus),
            Attribute::PublicExponent(exponent),
            Attribute::Encrypt(true),
            Attribute::Token(false),
            Attribute::Private(false),
        ];
        let pub_handle = match ctx.session().create_object(&attrs) {
            Ok(h) => h,
            Err(e) => { eprintln!("[cb_enc_cert] create_object: {}", e); return -1; }
        };

        // RSA-OAEP-SHA256 encrypt via token
        let oaep_params = PkcsOaepParams::new(
            MechanismType::SHA256,
            PkcsMgfType::MGF1_SHA256,
            PkcsOaepSource::empty(),
        );
        let mechanism = Mechanism::RsaPkcsOaep(oaep_params);
        let ciphertext = ctx.session().encrypt(&mechanism, pub_handle, pt_slice);

        // Always destroy session key object
        let _ = ctx.session().destroy_object(pub_handle);

        match ciphertext {
            Ok(ct) => {
                let buf_capacity = *ciphertext_len as usize;
                if ct.len() > buf_capacity {
                    eprintln!("[cb_enc_cert] buffer too small: need {}, have {}", ct.len(), buf_capacity);
                    return -1;
                }
                std::ptr::copy_nonoverlapping(ct.as_ptr(), ciphertext_out, ct.len());
                *ciphertext_len = ct.len() as u32;
                0
            }
            Err(e) => { eprintln!("[cb_enc_cert] token OAEP encrypt: {}", e); -1 }
        }
    });
    result.unwrap_or(-1)
}
```

### Step 3: Rewrite `cb_rsa_pss_verify`
Replace entire function body:
```rust
pub unsafe extern "C" fn cb_rsa_pss_verify(
    digest: *const u8, digest_len: u32,
    sig: *const u8, sig_len: u32,
    sender_cert_der: *const u8, sender_cert_der_len: u32,
    user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if digest.is_null() || sig.is_null() || sender_cert_der.is_null() || user_ctx.is_null() {
            return -1;
        }
        let ctx = &*(user_ctx as *const TokenContext);
        let digest_slice = slice::from_raw_parts(digest, digest_len as usize);
        let sig_slice = slice::from_raw_parts(sig, sig_len as usize);
        let cert_slice = slice::from_raw_parts(sender_cert_der, sender_cert_der_len as usize);

        // Extract sender's RSA public key components
        let (modulus, exponent) = match extract_rsa_key_components(cert_slice) {
            Ok(kc) => kc,
            Err(e) => { eprintln!("[cb_verify] key extract: {}", e); return -1; }
        };

        // Import sender public key as session object on token
        let attrs = vec![
            Attribute::Class(ObjectClass::PUBLIC_KEY),
            Attribute::KeyType(KeyType::Rsa),
            Attribute::Modulus(modulus),
            Attribute::PublicExponent(exponent),
            Attribute::Verify(true),
            Attribute::Token(false),
            Attribute::Private(false),
        ];
        let pub_handle = match ctx.session().create_object(&attrs) {
            Ok(h) => h,
            Err(e) => { eprintln!("[cb_verify] create_object: {}", e); return -1; }
        };

        // RSA-PSS-SHA256 verify via token (pre-hashed digest, salt=32 fixed)
        let pss_params = PkcsPssParams {
            hash_alg: MechanismType::SHA256,
            mgf: PkcsMgfType::MGF1_SHA256,
            s_len: 32_usize.try_into().expect("32 fits in Ulong"),
        };
        let mechanism = Mechanism::RsaPkcsPss(pss_params);
        let verify_result = ctx.session().verify(&mechanism, pub_handle, digest_slice, sig_slice);

        // Always destroy session key object
        let _ = ctx.session().destroy_object(pub_handle);

        match verify_result {
            Ok(()) => 0,
            Err(e) => { eprintln!("[cb_verify] PSS verify failed: {}", e); -1 }
        }
    });
    result.unwrap_or(-1)
}
```

### Step 4: Add `extract_rsa_key_components` helper
Replace `extract_spki_der` helper or add alongside:
```rust
/// Parse cert DER → extract RSA modulus + public exponent as big-endian bytes.
/// Used by both cb_rsa_oaep_enc_cert and cb_rsa_pss_verify for C_CreateObject.
fn extract_rsa_key_components(cert_der: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let (_, cert) = parse_x509_certificate(cert_der)
        .map_err(|e| format!("X.509 parse: {:?}", e))?;
    let spki_der = cert.public_key().raw.to_vec();
    let pub_key = RsaPublicKey::from_public_key_der(&spki_der)
        .map_err(|e| format!("RSA key parse: {}", e))?;
    let modulus = pub_key.n().to_bytes_be();
    let exponent = pub_key.e().to_bytes_be();
    Ok((modulus, exponent))
}
```

Note: `extract_spki_der` is no longer used after rewrite — remove it.

### Step 5: Clean up imports
- Remove `rsa::Oaep`, `rsa::sha2::Sha256`, `rand::thread_rng`
- Remove `rsa::pss::{Signature as PssSignature, VerifyingKey}`, `rsa::signature::hazmat::PrehashVerifier`
- Add `cryptoki::object::{Attribute, ObjectClass, KeyType}`
- Keep `rsa::pkcs8::DecodePublicKey`, `rsa::traits::PublicKeyParts`, `rsa::RsaPublicKey`
- Keep all existing `cryptoki::mechanism::*` imports (already used by sign/decrypt)

## Todo List
- [x] Add `cryptoki::object::{Attribute, ObjectClass, KeyType}` import
- [x] Remove software RSA imports (Oaep, Sha256, thread_rng, PssSignature, VerifyingKey, PrehashVerifier)
- [x] Add `extract_rsa_key_components()` helper fn (replaces `extract_spki_der`)
- [x] Remove `extract_spki_der` fn
- [x] Rewrite `cb_rsa_oaep_enc_cert`: C_CreateObject + C_Encrypt + destroy_object
- [x] Rewrite `cb_rsa_pss_verify`: C_CreateObject + C_Verify (salt=32) + destroy_object
- [x] `cargo check` — no errors in callbacks.rs

## Success Criteria
- `cargo check` passes with no errors
- No software RSA crypto in callbacks (only key component extraction via `rsa` crate)
- `destroy_object` always called even on encrypt/verify error

## Risk Assessment
<!-- Updated: Validation Session 1 - token untested, error logging only, no software RSA fallback -->
- **HIGH**: Some PKCS#11 tokens reject `C_Encrypt`/`C_Verify` on imported session objects (`CKA_TOKEN=false`). **Target token has NOT been tested for this.** Runtime-only issue — detectable in testing.
  - Mitigation: error logging only — `eprintln!` surfaces "CKR_KEY_FUNCTION_NOT_PERMITTED" or similar. No software RSA fallback. Document as known limitation.
  - No software RSA fallback will be added (user decision: accept risk).
- **MEDIUM**: PSS verify with fixed salt=32 will reject files signed by tokens that used max-salt. **Confirmed safe** — all counterparts use salt=32 (validated).
- **LOW**: `destroy_object` failure is silently ignored (`let _ = ...`) — acceptable since it's a session object that auto-cleans on session close.

## Security Considerations
- Session objects (`CKA_TOKEN=false`) are automatically destroyed when PKCS#11 session closes — no persistent key material left on token
- `destroy_object` called explicitly for immediate cleanup even before session close

## Next Steps
- Phase 4: remove `rand` dep, run full `cargo build` to verify

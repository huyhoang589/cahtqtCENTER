---
name: Brainstorm — Crypto API Migration + Full Token Hardware
description: Analysis for migrating to htqt-api.h v2 SF API and routing all 4 callbacks through PKCS#11 token hardware
type: project
---

# Brainstorm Report: Crypto API Migration + Full Token Hardware

## Problem Statement

Migrate encrypt/decrypt DLL calls to match `htqt-api.h` (SF type API), and route all 4 `CryptoCallbacksV2` callbacks through PKCS#11 token hardware.

## API Changes (htqt-api.h vs current)

| Function | Current | New |
|---|---|---|
| Encrypt | `encHTQT_sf_multi(BatchEncryptParams, CryptoCallbacksV2, BatchResult[], err, err_len)` | **Same — no change** |
| Decrypt | `decHTQT_sf(BatchSfDecryptParams, CryptoCallbacksV2, BatchResult[], err, err_len)` | `decrypt_one_sfv1(sf1_path, output_dir, CryptoCallbacksV2, flags, out_path_buf, out_path_buf_len, err_buf, err_len)` |

## Agreed Decisions

### Decrypt Loop Strategy
- Open **one PKCS#11 session** via `open_token_session`
- Loop N × `decrypt_one_sfv1` calls inside single `spawn_blocking`
- Per-file result (output_path buffer) handled individually

### Callback Hardware Routing
| Callback | Agreed Impl |
|---|---|
| `cb_rsa_pss_sign` | PKCS#11 `C_Sign` (already done, no change) |
| `cb_rsa_oaep_enc_cert` | `C_CreateObject` (import recipient pub key as session obj) → `C_Encrypt` via token |
| `cb_rsa_oaep_decrypt` | PKCS#11 `C_Decrypt` (already done, no change) |
| `cb_rsa_pss_verify` | `C_CreateObject` (import sender pub key as session obj) → `C_Verify` via token, fixed salt=32 |

### PSS Verify Salt
- Fixed `s_len = 32` for `C_Verify` 
- Risk accepted: files signed by tokens using max-salt PSS may fail verification

## Files to Change

1. `src-tauri/src/htqt_ffi/types.rs`
   - Remove: `FnDecHTQTSf`, `BatchSfDecryptParams`
   - Add: `FnDecryptOneSfv1` matching new C signature

2. `src-tauri/src/htqt_ffi/lib_loader.rs`
   - Change symbol: `decHTQT_sf` → `decrypt_one_sfv1`
   - New method: `decrypt_one_sfv1(&self, sf1_path, output_dir, cbs, flags) -> Result<String, String>`

3. `src-tauri/src/commands/decrypt.rs`
   - Remove `BatchSfDecryptParams` usage
   - One `open_token_session`, N-call loop, per-file result collection

4. `src-tauri/src/htqt_ffi/callbacks.rs`
   - `cb_rsa_oaep_enc_cert`: `_user_ctx` → `user_ctx` (cast to `TokenContext`), extract pub key from cert → `C_CreateObject` → `C_Encrypt` → `destroy_object`
   - `cb_rsa_pss_verify`: `_user_ctx` → `user_ctx`, extract pub key from sender cert → `C_CreateObject` → `C_Verify` (salt=32) → `destroy_object`

5. `src-tauri/Cargo.toml`
   - Remove `rand` crate (no longer needed after software RSA encrypt removed)
   - Keep `rsa` crate (still needed for extracting modulus/exponent bytes from DER for C_CreateObject)

## Risks

1. **Token support for session key objects**: Not all PKCS#11 implementations allow `C_Encrypt`/`C_Verify` on keys with `CKA_TOKEN=false`. Runtime validation required with actual hardware.
2. **PSS salt mismatch**: Fixed salt=32 for verify — incompatible with files signed by max-salt tokens.
3. **`user_ctx` non-null contract**: `cb_rsa_oaep_enc_cert` and `cb_rsa_pss_verify` now require non-null `user_ctx`. This is satisfied since both callbacks are only wired when `TokenContext` is available.

## Unresolved Questions

- None. All key decisions confirmed by user.

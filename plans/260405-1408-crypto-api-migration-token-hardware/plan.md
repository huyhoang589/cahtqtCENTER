---
title: "Crypto API Migration — htqt-api.h SF + Full Token Hardware"
description: "Migrate decrypt to decrypt_one_sfv1, route all 4 CryptoCallbacksV2 callbacks through PKCS#11 token hardware"
status: complete
priority: P1
effort: 4h
issue:
branch: feature/crypto.api.sf
tags: [backend, crypto, pkcs11, refactor]
created: 2026-04-05
completed: 2026-04-05
---

# Crypto API Migration — htqt-api.h SF + Full Token Hardware

## Overview

Migrate FFI layer to match `htqt-api.h` v2 SF format and route all 4 `CryptoCallbacksV2`
callbacks through PKCS#11 token hardware (replacing software RSA for encrypt-for-cert + verify).

**Brainstorm report:** `plans/reports/brainstorm-260405-1408-crypto-api-migration.md`
**Header reference:** `feature/1. api-sf-type/htqt-api.h`

## Phases

| # | Phase | Status | Effort | Link |
|---|-------|--------|--------|------|
| 1 | FFI types + lib_loader update | Complete | 1h | [phase-01](./phase-01-ffi-types-and-lib-loader.md) |
| 2 | Decrypt command — single session loop | Complete | 1h | [phase-02-decrypt-command-single-session-loop.md](./phase-02-decrypt-command-single-session-loop.md) |
| 3 | Callbacks — C_CreateObject token crypto | Complete | 1.5h | [phase-03-callbacks-token-crypto.md](./phase-03-callbacks-token-crypto.md) |
| 4 | Cleanup + compile verify | Complete | 0.5h | [phase-04-cleanup-and-compile-verify.md](./phase-04-cleanup-and-compile-verify.md) |

## Key Decisions (from brainstorm)

- **Decrypt API**: `decHTQT_sf(BatchSfDecryptParams)` → `decrypt_one_sfv1(sf1_path, output_dir, cbs, flags, ...)`
- **Session strategy**: One `open_token_session`, N calls inside single `spawn_blocking`
- **cb_rsa_oaep_enc_cert**: software RSA → `C_CreateObject` (recipient pub key, `CKA_TOKEN=false`) + `C_Encrypt`
- **cb_rsa_pss_verify**: software RSA → `C_CreateObject` (sender pub key) + `C_Verify` (PSS, salt=32 fixed)
- **cb_rsa_pss_sign + cb_rsa_oaep_decrypt**: no change (already PKCS#11)
- **Cargo.toml**: remove `rand` crate (keep `rsa` for key component extraction)

## Dependencies

- `cryptoki = "0.6"` — must support `session.create_object()`, `session.encrypt()`, `session.verify()`
- `rsa = "0.9"` — keep for extracting modulus/exponent bytes from DER certs

## Validation Log

### Session 1 — 2026-04-05
**Trigger:** Initial plan validation before implementation
**Questions asked:** 6

#### Questions & Answers

1. **[Risk]** Phase 3 marks HIGH risk: some PKCS#11 tokens reject C_Encrypt/C_Verify on session-imported keys (CKA_TOKEN=false). Has the target token been tested for this, and what's the fallback strategy?
   - Options: Token tested, confirmed OK | Untested — add error logging only | Untested — add software RSA fallback
   - **Answer:** Untested — add error logging only
   - **Rationale:** Token compatibility unverified at plan time. Accept the risk; surface clear error (CKR_KEY_FUNCTION_NOT_PERMITTED) via eprintln. No software RSA fallback needed. Document as known limitation.

2. **[Assumptions]** Phase 3 removes the 2-salt PSS verify retry (salt=32 then max_salt) and fixes salt=32 only. Are you certain all files in use were signed with salt=32?
   - Options: Yes, salt=32 is standard here | Uncertain — keep 2-salt retry | Uncertain — token controls salt
   - **Answer:** Yes, salt=32 is standard here
   - **Rationale:** Fixed salt=32 is confirmed safe. Removing 2-salt retry is intentional. Max-salt signed files are not in scope.

3. **[Architecture]** Phase 2 sets verify_fn: Some(cb_rsa_pss_verify) in the decrypt CryptoCallbacksV2. Does decrypt actually require PSS signature verification?
   - Options: Yes — DLL calls verify during decrypt | No — set verify_fn: None | Unknown — check docs
   - **Answer:** Yes — DLL calls verify during decrypt
   - **Rationale:** SF v1 format includes a sender signature verified at decrypt time. verify_fn must be set. Plan is correct.

4. **[Architecture]** Phase 2 acquires DLL_LOCK inside each decrypt_one_sfv1 call (per-file). Is per-call locking acceptable?
   - Options: Per-call lock is fine | Hold lock across full loop
   - **Answer:** Per-call lock is fine
   - **Rationale:** spawn_blocking runs single-threaded. Per-call lock matches existing pattern. No change needed.

5. **[Tradeoffs]** Phase 2 passes HTQT_BATCH_CONTINUE_ON_ERROR flag to each decrypt_one_sfv1 call. DLL may ignore it for single-file API. Pass anyway or use flags=0?
   - Options: Pass HTQT_BATCH_CONTINUE_ON_ERROR anyway | Pass flags=0
   - **Answer:** Pass HTQT_BATCH_CONTINUE_ON_ERROR anyway
   - **Rationale:** Harmless if ignored. Errors are already isolated by Result return. Plan is correct.

6. **[Risks]** Phase 4 removes rand from Cargo.toml. Is rand used only by crypto callbacks or possibly elsewhere?
   - Options: Only used by callbacks — safe to remove | Unsure — check Cargo.toml before removing
   - **Answer:** Unsure — check Cargo.toml before removing
   - **Rationale:** Must run `cargo tree | grep rand` before removing to avoid breaking transitive deps.

#### Confirmed Decisions
- Token compat risk: accepted — error logging only, no software RSA fallback
- PSS salt=32: confirmed standard — 2-salt retry removal is safe
- verify_fn in decrypt: confirmed required — DLL verifies SF envelope signature
- DLL_LOCK strategy: per-call locking confirmed adequate
- flags value: pass HTQT_BATCH_CONTINUE_ON_ERROR confirmed
- rand removal: must verify with cargo tree first

#### Action Items
- [ ] Phase 3: document token untested risk in Risk Assessment — error logging only, no fallback
- [ ] Phase 4: add `cargo tree | grep rand` step before removing rand from Cargo.toml

#### Impact on Phases
- Phase 3: Risk Assessment — clarify "untested" status; error logging only (no software RSA fallback)
- Phase 4: Implementation Steps — add cargo tree check before rand removal

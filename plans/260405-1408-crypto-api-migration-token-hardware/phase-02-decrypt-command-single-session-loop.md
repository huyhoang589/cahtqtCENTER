# Phase 2 — Decrypt Command: Single Session Loop

**Context:** `plans/reports/brainstorm-260405-1408-crypto-api-migration.md` | `commands/decrypt.rs`
**Depends on:** Phase 1 (FnDecryptOneSfv1 + lib_loader method)

## Overview
- **Priority:** P1
- **Status:** Complete
- **Effort:** 1h
- Rewrite `run_decrypt_batch` to open one PKCS#11 session then loop N × `decrypt_one_sfv1` calls, collecting per-file results.

## Key Insights
- Old: one `decHTQT_sf(BatchSfDecryptParams, ...)` call → batch results slice
- New: one session open + N × `lib.decrypt_one_sfv1(sf1_path, output_dir, &cbs, flags)` calls
- `BatchResult` struct and `BatchSfDecryptParams` no longer needed — results collected per-call
- `sign_fn` and `rsa_enc_cert_fn` remain `None` for decrypt (DLL doesn't call them)
- Progress events still emit per-file via `cb_progress` if wired; decrypt currently uses `progress_fn: None`
- `HTQT_BATCH_CONTINUE_ON_ERROR` flag now passed per-file call

## Related Code Files
- **Modify:** `src-tauri/src/commands/decrypt.rs`

## Implementation Steps

1. Remove `BatchSfDecryptParams` import from `use` block
2. Keep all pre-`spawn_blocking` logic unchanged (token login check, own_cert_der, output_dir resolution)
3. Inside `spawn_blocking`, replace the old batch flow with:

```rust
// Build CStrings for output_dir (shared) and per-file paths
let output_dir_cstring = CString::new(output_dir_str_clone.as_str())
    .map_err(|e| e.to_string())?;

let input_cstrings: Vec<CString> = file_paths_owned.iter()
    .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
    .collect::<Result<_, _>>()?;

// Open ONE PKCS#11 session for all decrypt calls
let ctx = open_token_session(
    &pkcs11_lib, slot_id, &pin_str,
    app_clone, own_cert_der_clone.clone(),
    "decrypt-progress".to_string(),
)?;
let ctx_box = Box::new(ctx);
let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

let cbs = CryptoCallbacksV2 {
    sign_fn: None,
    rsa_enc_cert_fn: None,
    rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
    verify_fn: Some(callbacks::cb_rsa_pss_verify),
    progress_fn: None,
    user_ctx: user_ctx_ptr,
    own_cert_der: if own_cert_der_clone.is_empty() { ptr::null() } else { own_cert_der_clone.as_ptr() },
    own_cert_der_len: own_cert_der_clone.len() as u32,
    reserved: [ptr::null_mut(); 3],
};

// Loop: one decrypt_one_sfv1 call per file
let guard = crate::lock_helper::safe_lock(&htqt_lib_arc)?;
let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;

let mut file_results: Vec<(usize, Result<String, String>)> = Vec::with_capacity(file_count);
for (i, cstr) in input_cstrings.iter().enumerate() {
    let result = lib.decrypt_one_sfv1(
        cstr.as_ptr(),
        output_dir_cstring.as_ptr(),
        &cbs,
        HTQT_BATCH_CONTINUE_ON_ERROR,
    );
    file_results.push((i, result));
}
drop(guard);
drop(ctx_box); // closes session

Ok(file_results)
```

4. Update post-`spawn_blocking` result processing:
   - `dec_results` is now `Vec<(usize, Result<String, String>)>` instead of `Vec<BatchResult>`
   - Loop: `(i, result)` — `i` = file index, `result` = Ok(output_path) or Err(msg)
   - Success: `success_count += 1`, emit `decrypt-progress` with `status: "success"`
   - Error: `error_count += 1`, emit with `status: "error"`, push to `errors`
   - Log via `logs_repo::insert_log` same as before

5. Remove unused `BatchResult` from imports

## Todo List
- [x] Remove `BatchSfDecryptParams` + `BatchResult` from imports
- [x] Replace batch build (FileEntry slice, BatchSfDecryptParams) with per-file CString loop
- [x] Replace single `lib.dec_sf()` call with per-file `lib.decrypt_one_sfv1()` loop
- [x] Update result type from `Vec<BatchResult>` to `Vec<(usize, Result<String, String>)>`
- [x] Update post-spawn result processing loop to match new type
- [x] `cargo check` — no errors in commands/decrypt.rs

## Success Criteria
- `cargo check` passes with no errors in decrypt command
- Per-file error isolation: one file failure doesn't abort remaining files

## Risk Assessment
- DLL_LOCK held per-file (inside `lib.decrypt_one_sfv1` which acquires lock internally) — verify lock is acquired/released per call, not held across all N calls. If lock is held for full loop, it's fine (single thread).
- `HTQT_BATCH_CONTINUE_ON_ERROR` flag behavior with `decrypt_one_sfv1` — DLL may ignore it for single-file calls; error is returned as non-zero rc regardless.

## Next Steps
- Phase 3: rewrite callbacks for PKCS#11 token crypto

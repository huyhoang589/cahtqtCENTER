# Phase 3: Decrypt Command

**Status:** pending  
**Priority:** high  
**Depends on:** Phase 1  
**File:** `src-tauri/src/commands/decrypt.rs`

## Context Links
- Phase 1: `phase-01-ffi-types-and-loader.md`
- Current impl: `src-tauri/src/commands/decrypt.rs`
- Header: `feature/1. api-sf-type/htqt-api.h` — `BatchSfDecryptParams`, `decHTQT_sf`

## Overview

`decHTQT_v2` was a per-file call with explicit `sf_path`, `output_path`, `recipient_id`.
New `decHTQT_sf` takes `BatchSfDecryptParams*` — batch all files in one call.
DLL derives output filenames from `orig_name` in SF header; we read them from `BatchResult.output_path`.
`recipient_id` removed from command — fingerprint matching via `own_cert_der` in `CryptoCallbacksV2`.

## Changes

### 1. Remove `recipient_id` from token state read

```rust
// BEFORE
let (pkcs11_lib, slot_id, pin_str, recipient_id) = {
    ...
    let cert_cn = login.cert_cn.clone().ok_or("Token cert_cn not available")?;
    (pkcs11_lib, slot_id, pin, cert_cn)
};

// AFTER — keep cert_cn for log message only (not passed to DLL)
let (pkcs11_lib, slot_id, pin_str, cert_cn_log) = {
    ...
    let cert_cn = login.cert_cn.clone().unwrap_or_default(); // for log only
    (pkcs11_lib, slot_id, pin, cert_cn)
};
```

### 2. Replace per-file loop with single batch call

```rust
// BEFORE: loop calling lib.dec_v2() per file
let mut results = Vec::with_capacity(file_paths_owned.len());
for file_path in &file_paths_owned {
    let stem = ...;
    let dst_str = format!("{}/{}", output_dir_str_clone, stem);
    let dec_result = lib.dec_v2(file_path, &dst_str, &recipient_id_clone, &cbs);
    results.push((file_path.clone(), dst_str, dec_result));
}

// AFTER: build BatchSfDecryptParams, single dec_sf() call
let input_cstrings: Vec<CString> = file_paths_owned.iter()
    .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
    .collect::<Result<_, _>>()?;
// file_id reuses stem for result tracking
let file_id_cstrings: Vec<CString> = file_paths_owned.iter()
    .map(|p| {
        let stem = Path::new(p).file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        CString::new(stem).map_err(|e| e.to_string())
    })
    .collect::<Result<_, _>>()?;
let output_dir_cstring = CString::new(output_dir_str_clone.as_str())
    .map_err(|e| e.to_string())?;

let file_entries: Vec<FileEntry> = (0..file_count)
    .map(|i| FileEntry {
        input_path: input_cstrings[i].as_ptr(),
        file_id: file_id_cstrings[i].as_ptr(),
    })
    .collect();

let params = BatchSfDecryptParams {
    files: file_entries.as_ptr(),
    file_count: file_count as u32,
    output_dir: output_dir_cstring.as_ptr(),
    flags: HTQT_BATCH_CONTINUE_ON_ERROR,
    reserved: [ptr::null_mut(); 2],
};

let mut batch_results: Vec<BatchResult> = (0..file_count)
    .map(|_| BatchResult::default())
    .collect();

let guard = crate::lock_helper::safe_lock(&htqt_lib_arc)?;
let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;
lib.dec_sf(&params, &cbs, &mut batch_results)?;
drop(guard);
drop(ctx_box);

Ok(batch_results)
```

### 3. Update spawn_blocking return type

```rust
// BEFORE
tokio::task::spawn_blocking(move || -> Result<Vec<(String, String, Result<(), String>)>, String>

// AFTER
tokio::task::spawn_blocking(move || -> Result<Vec<BatchResult>, String>
```

Add to imports at top of file:
```rust
use crate::htqt_ffi::{
    callbacks,
    error_codes::{HTQT_BATCH_CONTINUE_ON_ERROR, HTQT_OK},
    htqt_error_message, htqt_error_name,
    token_context::open_token_session,
    BatchResult, BatchSfDecryptParams, CryptoCallbacksV2, FileEntry,
};
```

### 4. Update result loop — read output_path from BatchResult

```rust
// BEFORE: iterates Vec<(file_path, dst_str, Result<(), String>)>
for (i, (file_path, dst_str, result)) in dec_results.iter().enumerate() {

// AFTER: iterates Vec<BatchResult>
for (i, result) in dec_results.iter().enumerate() {
    let fi = result.file_index as usize;
    let file_path = file_paths.get(fi).map(String::as_str).unwrap_or("?");
    // output_path filled by DLL from SF header orig_name
    let output_path = crate::ffi_helpers::string_from_c_buf(&result.output_path);
    // fallback: if DLL didn't fill it, use output_dir
    let dst_for_log = if output_path.is_empty() { &output_dir_str } else { &output_path };

    let file_name = Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    let (status_str, error_msg) = if result.status == HTQT_OK {
        success_count += 1;
        ("success".to_string(), None)
    } else {
        error_count += 1;
        let name = htqt_error_name(result.status);
        let message = htqt_error_message(result.status);
        let detail = crate::ffi_helpers::string_from_c_buf(&result.error_detail);
        let error_str = if detail.is_empty() {
            format!("[{}] {}: {}", result.status, name, message)
        } else {
            format!("[{}] {}: {} — {}", result.status, name, message, detail)
        };
        emit_app_log(app, "error", &format!("[DECRYPT] {}: {}", file_name, error_str));
        errors.push(format!("{}: {}", file_name, error_str));
        ("error".to_string(), Some(error_str))
    };

    let _ = app.emit("decrypt-progress", DecryptProgress {
        current: i + 1,
        total,
        file_name: file_name.clone(),
        file_path: file_path.to_string(),
        status: status_str.clone(),
        error: error_msg.clone(),
    });

    let _ = logs_repo::insert_log(
        &state.db, "DECRYPT",
        file_path, dst_for_log,
        None, &status_str,
        error_msg.as_deref(),
    ).await;
}
```

### 5. Import cleanup

Remove unused imports after changes:
- `use std::ffi::CString` — still needed (for building CStrings)
- `recipient_id` references — remove

## Todo

- [ ] Update token state read — remove `recipient_id`, keep `cert_cn_log`
- [ ] Replace per-file loop with `BatchSfDecryptParams` + single `dec_sf()` call
- [ ] Update `spawn_blocking` return type to `Vec<BatchResult>`
- [ ] Update result iteration loop to use `BatchResult` fields
- [ ] Log `output_path` from `BatchResult`, fallback to `output_dir`
- [ ] Add `BatchSfDecryptParams`, `FileEntry` to imports
- [ ] `cargo check` — no errors

## Success Criteria

- Single `dec_sf()` call replaces per-file loop
- `recipient_id` not passed to DLL
- Output path logged from `BatchResult.output_path`
- `cargo check` clean

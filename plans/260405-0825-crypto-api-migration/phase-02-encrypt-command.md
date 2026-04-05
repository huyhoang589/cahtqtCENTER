# Phase 2: Encrypt Command

**Status:** pending  
**Priority:** high  
**Depends on:** Phase 1  
**File:** `src-tauri/src/commands/encrypt.rs`

## Context Links
- Phase 1: `phase-01-ffi-types-and-loader.md`
- Current impl: `src-tauri/src/commands/encrypt.rs`

## Overview

New `encHTQT_sf_multi` produces **one `.sf1` per input file** (all N recipients embedded).
Results array is now `file_count` entries, not `file_count * recipient_count`.
Progress events shift from per-(file,recipient) pair → per-file.

## Changes

### 1. Results vec capacity

```rust
// BEFORE
let mut batch_results: Vec<BatchResult> = (0..total_pairs)
    .map(|_| BatchResult::default())
    .collect();

// AFTER
let mut batch_results: Vec<BatchResult> = (0..file_count)
    .map(|_| BatchResult::default())
    .collect();
```

### 2. DLL call — method rename

```rust
// BEFORE
lib.enc_multi(&params, &cbs, &mut batch_results)?;

// AFTER  (same method, field renamed internally in Phase 1)
lib.enc_multi(&params, &cbs, &mut batch_results)?;
// NOTE: enc_multi() signature unchanged — no call-site change needed
```

> `enc_multi()` public signature stays the same. Only internal field/transmute type changes in lib_loader.rs (Phase 1).

### 3. Result loop — per-file progress

```rust
// BEFORE: iterates total_pairs, uses result.recipient_index
for (pair_idx, result) in batch_results.iter().enumerate() {
    let fi = result.file_index as usize;
    let ri = result.recipient_index as usize;
    // emits current: pair_idx + 1 / total: total_files
    ...
}

// AFTER: iterates file_count, no recipient_index needed
for result in batch_results.iter() {
    let fi = result.file_index as usize;
    let file_path_str = src_paths.get(fi).map(String::as_str).unwrap_or("?");
    let output_path = crate::ffi_helpers::string_from_c_buf(&result.output_path);

    let file_name = Path::new(file_path_str)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_path_str.to_string());

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
        errors.push(format!("{}: {}", file_name, error_str));
        ("error".to_string(), Some(error_str))
    };

    let _ = app.emit("encrypt-progress", EncryptProgress {
        current: fi + 1,
        total: file_count,
        file_name: file_name.clone(),
        file_path: file_path_str.to_string(),
        status: status_str.clone(),
        error: error_msg.clone(),
    });

    let _ = logs_repo::insert_log(
        &state.db,
        "ENCRYPT",
        file_path_str,
        &output_path,
        None,
        &status_str,
        error_msg.as_deref(),
    ).await;
}
```

### 4. Remove `total_pairs` variable entirely

<!-- Updated: Validation Session 1 - Remove total_pairs entirely, drop >10k warning guard too -->
`total_pairs` is no longer needed. Remove it and the associated >10k batch warning log entirely.

```rust
// Remove: let total_pairs = file_count * recip_count;
// Remove: if total_pairs > 10_000 { warn!(...) }
```

### 5. `EncryptResult.total`

```rust
// BEFORE
let total = batch_results.len();  // was total_pairs

// AFTER — same, now equals file_count
let total = batch_results.len();
```

> No change needed — `batch_results.len()` is now `file_count` automatically.

## Todo

- [ ] Change results vec allocation to `file_count`
- [ ] Rewrite result iteration loop (per-file, drop `recipient_index`)
- [ ] Remove `total_pairs` and >10k warning log entirely
- [ ] `cargo check` — no errors

## Success Criteria

- Results vec has `file_count` capacity
- Progress events fired once per file (not per pair)
- No `recipient_index` used in event payload
- `cargo check` clean

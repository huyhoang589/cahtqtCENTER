# Phase 1 — FFI Types + lib_loader Update

**Context:** `plans/reports/brainstorm-260405-1408-crypto-api-migration.md` | `htqt_ffi/types.rs` | `htqt_ffi/lib_loader.rs`

## Overview
- **Priority:** P1
- **Status:** Complete
- **Effort:** 1h
- Remove batch decrypt types, add `FnDecryptOneSfv1` matching new header signature. Update lib_loader to load `decrypt_one_sfv1` symbol.

## Key Insights
- New header has `decrypt_one_sfv1` (single-file) replacing `decHTQT_sf` (batch with `BatchSfDecryptParams`)
- `encHTQT_sf_multi` signature is identical — no change needed
- `BatchSfDecryptParams` is no longer referenced after decrypt command update (Phase 2)

## Requirements
- `FnDecryptOneSfv1` must exactly mirror C signature from `htqt-api.h`:
  ```c
  int decrypt_one_sfv1(
      const char *sf1_path, const char *output_dir,
      const CryptoCallbacksV2 *cbs, uint32_t flags,
      char *out_path_buf, int out_path_buf_len,
      char *err_buf, int err_len)
  ```
- Remove all dead code (`FnDecHTQTSf`, `BatchSfDecryptParams`) — no unused types

## Related Code Files
- **Modify:** `src-tauri/src/htqt_ffi/types.rs`
- **Modify:** `src-tauri/src/htqt_ffi/lib_loader.rs`

## Implementation Steps

### `htqt_ffi/types.rs`

1. Remove `FnDecHTQTSf` type alias (lines ~65-71)
2. Remove `BatchSfDecryptParams` struct and its `unsafe impl Send/Sync` (lines ~132-142)
3. Add new type alias after `FnEncHTQTSfMulti`:
   ```rust
   /// decrypt_one_sfv1: decrypt a single SF v1 file using callback-based crypto.
   pub type FnDecryptOneSfv1 = unsafe extern "C" fn(
       sf1_path: *const c_char,
       output_dir: *const c_char,
       cbs: *const CryptoCallbacksV2,
       flags: c_uint,
       out_path_buf: *mut c_char,
       out_path_buf_len: c_int,
       err_buf: *mut c_char,
       err_len: c_int,
   ) -> c_int;
   ```

### `htqt_ffi/lib_loader.rs`

4. Rename field `dec_sf_fn` → `decrypt_one_sfv1_fn` in `HtqtLib` struct
5. In `HtqtLib::load()`: change symbol lookup from `b"decHTQT_sf\0"` → `b"decrypt_one_sfv1\0"`, update error message and transmute type to `FnDecryptOneSfv1`
6. Remove `dec_sf()` method entirely
7. Add new method:
   ```rust
   /// Decrypt a single SF v1 file. Returns output file path on success.
   pub fn decrypt_one_sfv1(
       &self,
       sf1_path: *const c_char,
       output_dir: *const c_char,
       cbs: &CryptoCallbacksV2,
       flags: u32,
   ) -> Result<String, String> {
       let mut out_path_buf = [0i8; 512];
       let mut err_buf = [0i8; 512];
       let _guard = DLL_LOCK.lock().map_err(|_| "DLL_LOCK poisoned".to_string())?;
       let rc = unsafe {
           let f: FnDecryptOneSfv1 = std::mem::transmute(self.decrypt_one_sfv1_fn);
           f(sf1_path, output_dir, cbs, flags,
             out_path_buf.as_mut_ptr(), 512,
             err_buf.as_mut_ptr(), 512)
       };
       if rc != 0 {
           let msg = crate::ffi_helpers::string_from_c_buf(&err_buf);
           Err(format!("decrypt_one_sfv1 failed ({}): {}", rc, msg))
       } else {
           Ok(crate::ffi_helpers::string_from_c_buf(&out_path_buf))
       }
   }
   ```

## Todo List
- [x] Remove `FnDecHTQTSf` from types.rs
- [x] Remove `BatchSfDecryptParams` + its unsafe impls from types.rs
- [x] Add `FnDecryptOneSfv1` type alias to types.rs
- [x] Rename `dec_sf_fn` → `decrypt_one_sfv1_fn` in lib_loader.rs struct
- [x] Update `load()`: symbol `decrypt_one_sfv1`, transmute to `FnDecryptOneSfv1`
- [x] Remove `dec_sf()` method from lib_loader.rs
- [x] Add `decrypt_one_sfv1()` method to lib_loader.rs
- [x] `cargo check` — verify no compile errors in htqt_ffi module

## Success Criteria
- `cargo check` passes on `htqt_ffi` module (ignore decrypt.rs errors until Phase 2)
- No references to `BatchSfDecryptParams` or `FnDecHTQTSf` remain

## Risk Assessment
- Low risk — pure type/symbol rename, no logic changes
- If `decrypt_one_sfv1` symbol not found at runtime → `HtqtLib::load()` returns error; app will surface "DLL symbol not found" on startup

## Next Steps
- Phase 2: update `commands/decrypt.rs` to use new `lib.decrypt_one_sfv1()` method

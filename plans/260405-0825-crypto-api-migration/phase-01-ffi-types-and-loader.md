# Phase 1: FFI Types + Loader

**Status:** pending  
**Priority:** high — blocks phases 2 & 3  
**Files:** `src-tauri/src/htqt_ffi/types.rs`, `src-tauri/src/htqt_ffi/lib_loader.rs`

## Context Links
- Header: `feature/1. api-sf-type/htqt-api.h`
- Brainstorm: `plans/reports/brainstorm-260405-0825-crypto-api-migration.md`

## Changes

### `htqt_ffi/types.rs`

**1. Rename fn pointer type** (same signature, new name):
```rust
// BEFORE
pub type FnEncHTQTMulti = unsafe extern "C" fn(...) -> c_int;
pub type FnDecHTQTV2 = unsafe extern "C" fn(
    sf_path: *const c_char,
    output_path: *const c_char,
    recipient_id: *const c_char,
    cbs: *const CryptoCallbacksV2,
    error_msg: *mut c_char,
    error_len: c_int,
) -> c_int;

// AFTER
pub type FnEncHTQTSfMulti = unsafe extern "C" fn(
    params: *const BatchEncryptParams,
    cbs: *const CryptoCallbacksV2,
    results: *mut BatchResult,
    error_msg: *mut c_char,
    error_len: c_int,
) -> c_int;

pub type FnDecHTQTSf = unsafe extern "C" fn(
    params: *const BatchSfDecryptParams,
    cbs: *const CryptoCallbacksV2,
    results: *mut BatchResult,
    error_msg: *mut c_char,
    error_len: c_int,
) -> c_int;
```

**2. Add `BatchSfDecryptParams` struct** (after `BatchEncryptParams`):
```rust
/// Batch decrypt parameters for SF v1 files.
#[repr(C)]
pub struct BatchSfDecryptParams {
    pub files: *const FileEntry,   // input_path = .sf1 file; file_id for result tracking
    pub file_count: u32,
    pub output_dir: *const c_char, // filenames taken from orig_name in SF header
    pub flags: u32,
    pub reserved: [*mut c_void; 2], // must be NULL
}

unsafe impl Send for BatchSfDecryptParams {}
unsafe impl Sync for BatchSfDecryptParams {}
```

### `htqt_ffi/lib_loader.rs`

**1. Update struct fields:**
```rust
pub struct HtqtLib {
    #[allow(dead_code)]
    lib: Library,
    enc_sf_multi_fn: *const (),   // was: enc_multi_fn
    dec_sf_fn: *const (),          // was: dec_v2_fn
    #[allow(dead_code)]
    get_error_fn: *const (),
}
```

**2. Update `load()` symbol resolution:**
```rust
// Symbol: "encHTQT_sf_multi\0"  (was "encHTQT_multi\0")
let enc_sf_multi_fn = unsafe {
    let sym: Symbol<FnEncHTQTSfMulti> = lib
        .get(b"encHTQT_sf_multi\0")
        .map_err(|_| "Symbol 'encHTQT_sf_multi' not found in htqt_crypto.dll".to_string())?;
    *sym as *const ()
};

// Symbol: "decHTQT_sf\0"  (was "decHTQT_v2\0")
let dec_sf_fn = unsafe {
    let sym: Symbol<FnDecHTQTSf> = lib
        .get(b"decHTQT_sf\0")
        .map_err(|_| "Symbol 'decHTQT_sf' not found in htqt_crypto.dll".to_string())?;
    *sym as *const ()
};

Ok(HtqtLib { lib, enc_sf_multi_fn, dec_sf_fn, get_error_fn })
```

**3. Update `enc_multi()` method:**
- Rename field `self.enc_multi_fn` → `self.enc_sf_multi_fn`
- Transmute to `FnEncHTQTSfMulti` (was `FnEncHTQTMulti`)
- Update doc comment: `results slice must have capacity >= file_count`

**4. Replace `dec_v2()` with `dec_sf()`:**
```rust
/// Batch decrypt SF v1 files via decHTQT_sf.
/// results slice must have capacity >= file_count.
pub fn dec_sf(
    &self,
    params: &BatchSfDecryptParams,
    cbs: &CryptoCallbacksV2,
    results: &mut [BatchResult],
) -> Result<i32, String> {
    let mut err_buf = [0i8; 512];
    let _guard = DLL_LOCK.lock().map_err(|_| "DLL_LOCK poisoned".to_string())?;

    let rc = unsafe {
        let f: FnDecHTQTSf = std::mem::transmute(self.dec_sf_fn);
        f(params, cbs, results.as_mut_ptr(), err_buf.as_mut_ptr(), 512)
    };

    if rc < 0 {
        let msg = crate::ffi_helpers::string_from_c_buf(&err_buf);
        Err(format!("decHTQT_sf failed ({}): {}", rc, msg))
    } else {
        Ok(rc)
    }
}
```

## Todo

- [ ] Rename `FnEncHTQTMulti` → `FnEncHTQTSfMulti` in types.rs
- [ ] Remove `FnDecHTQTV2`, add `FnDecHTQTSf` in types.rs
- [ ] Add `BatchSfDecryptParams` struct in types.rs
- [ ] Update `HtqtLib` struct fields in lib_loader.rs
- [ ] Update `load()`: new symbol strings + new fn pointer types
- [ ] Update `enc_multi()`: field rename + transmute type
- [ ] Replace `dec_v2()` with `dec_sf()` in lib_loader.rs
- [ ] `cargo check` — no errors

## Success Criteria

- `cargo check` passes clean
- `HtqtLib::load()` resolves both new symbols or returns descriptive `Err`
- `dec_sf()` accepts `BatchSfDecryptParams` + `&mut [BatchResult]`

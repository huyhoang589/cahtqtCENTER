use libloading::{os::windows::Library as WinLib, Library, Symbol};

use super::types::*;
use super::DLL_LOCK;

/// htqt_crypto v2 DLL wrapper — resolves 3 symbols: encHTQT_sf_multi, decHTQT_sf, HTQT_GetError.
pub struct HtqtLib {
    #[allow(dead_code)]
    lib: Library, // kept alive so raw fn pointers remain valid
    enc_sf_multi_fn: *const (),
    dec_sf_fn: *const (),
    #[allow(dead_code)]
    get_error_fn: *const (),
}

// Accessed through Arc<Mutex<Option<HtqtLib>>> in AppState — safe to mark.
unsafe impl Send for HtqtLib {}
unsafe impl Sync for HtqtLib {}

impl HtqtLib {
    /// Load htqt_crypto.dll from path and resolve v2 symbols.
    /// Uses LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR (0x100) so the DLL's own dependencies
    /// are found in the same directory, plus LOAD_LIBRARY_SEARCH_DEFAULT_DIRS (0x1000)
    /// for system DLLs.
    pub fn load(path: &str) -> Result<Self, String> {
        const LOAD_FLAGS: u32 = 0x0000_0100 | 0x0000_1000;
        let lib: Library = unsafe {
            WinLib::load_with_flags(path, LOAD_FLAGS)
                .map(Library::from)
                .map_err(|e| format!("Failed to load htqt_crypto.dll: {}", e))?
        };

        let enc_sf_multi_fn = unsafe {
            let sym: Symbol<FnEncHTQTSfMulti> = lib
                .get(b"encHTQT_sf_multi\0")
                .map_err(|_| "Symbol 'encHTQT_sf_multi' not found in htqt_crypto.dll".to_string())?;
            *sym as *const ()
        };

        let dec_sf_fn = unsafe {
            let sym: Symbol<FnDecHTQTSf> = lib
                .get(b"decHTQT_sf\0")
                .map_err(|_| "Symbol 'decHTQT_sf' not found in htqt_crypto.dll".to_string())?;
            *sym as *const ()
        };

        let get_error_fn = unsafe {
            let sym: Symbol<FnGetError> = lib
                .get(b"HTQT_GetError\0")
                .map_err(|_| "Symbol 'HTQT_GetError' not found in htqt_crypto.dll".to_string())?;
            *sym as *const ()
        };

        Ok(HtqtLib { lib, enc_sf_multi_fn, dec_sf_fn, get_error_fn })
    }

    /// Batch encrypt M files × N recipients via encHTQT_sf_multi.
    /// results slice must have capacity >= file_count.
    /// Returns Ok(rc): 0 = all success, >0 = partial failures in results.
    pub fn enc_multi(
        &self,
        params: &BatchEncryptParams,
        cbs: &CryptoCallbacksV2,
        results: &mut [BatchResult],
    ) -> Result<i32, String> {
        let mut err_buf = [0i8; 512];
        let _guard = DLL_LOCK.lock().map_err(|_| "DLL_LOCK poisoned".to_string())?;

        let rc = unsafe {
            let f: FnEncHTQTSfMulti = std::mem::transmute(self.enc_sf_multi_fn);
            f(params, cbs, results.as_mut_ptr(), err_buf.as_mut_ptr(), 512)
        };

        if rc < 0 {
            let msg = crate::ffi_helpers::string_from_c_buf(&err_buf);
            Err(format!("encHTQT_sf_multi failed ({}): {}", rc, msg))
        } else {
            Ok(rc)
        }
    }

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
}

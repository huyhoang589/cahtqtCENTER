use std::ffi::CString;

use libloading::{os::windows::Library as WinLib, Library, Symbol};

use super::types::*;
use super::DLL_LOCK;

/// htqt_crypto v2 DLL wrapper — resolves 3 symbols: encHTQT_multi, decHTQT_v2, HTQT_GetError.
pub struct HtqtLib {
    #[allow(dead_code)]
    lib: Library, // kept alive so raw fn pointers remain valid
    enc_multi_fn: *const (),
    dec_v2_fn: *const (),
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

        let enc_multi_fn = unsafe {
            let sym: Symbol<FnEncHTQTMulti> = lib
                .get(b"encHTQT_multi\0")
                .map_err(|_| "Symbol 'encHTQT_multi' not found in htqt_crypto.dll".to_string())?;
            *sym as *const ()
        };

        let dec_v2_fn = unsafe {
            let sym: Symbol<FnDecHTQTV2> = lib
                .get(b"decHTQT_v2\0")
                .map_err(|_| "Symbol 'decHTQT_v2' not found in htqt_crypto.dll".to_string())?;
            *sym as *const ()
        };

        let get_error_fn = unsafe {
            let sym: Symbol<FnGetError> = lib
                .get(b"HTQT_GetError\0")
                .map_err(|_| "Symbol 'HTQT_GetError' not found in htqt_crypto.dll".to_string())?;
            *sym as *const ()
        };

        Ok(HtqtLib { lib, enc_multi_fn, dec_v2_fn, get_error_fn })
    }

    /// Batch encrypt M files × N recipients via encHTQT_multi.
    /// results slice must have capacity >= file_count * recipient_count.
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
            let f: FnEncHTQTMulti = std::mem::transmute(self.enc_multi_fn);
            f(params, cbs, results.as_mut_ptr(), err_buf.as_mut_ptr(), 512)
        };

        if rc < 0 {
            let msg = crate::ffi_helpers::string_from_c_buf(&err_buf);
            Err(format!("encHTQT_multi failed ({}): {}", rc, msg))
        } else {
            Ok(rc)
        }
    }

    /// Decrypt a single SF file via decHTQT_v2.
    pub fn dec_v2(
        &self,
        sf_path: &str,
        output_path: &str,
        recipient_id: &str,
        cbs: &CryptoCallbacksV2,
    ) -> Result<(), String> {
        let sf = CString::new(sf_path).map_err(|e| e.to_string())?;
        let out = CString::new(output_path).map_err(|e| e.to_string())?;
        let rid = CString::new(recipient_id).map_err(|e| e.to_string())?;
        let mut err_buf = [0i8; 512];

        let _guard = DLL_LOCK.lock().map_err(|_| "DLL_LOCK poisoned".to_string())?;

        let rc = unsafe {
            let f: FnDecHTQTV2 = std::mem::transmute(self.dec_v2_fn);
            f(sf.as_ptr(), out.as_ptr(), rid.as_ptr(), cbs, err_buf.as_mut_ptr(), 512)
        };

        if rc != 0 {
            let msg = crate::ffi_helpers::string_from_c_buf(&err_buf);
            Err(format!("decHTQT_v2 failed ({}): {}", rc, msg))
        } else {
            Ok(())
        }
    }
}

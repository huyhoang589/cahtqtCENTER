use std::ffi::{c_char, c_int, c_uint, c_void};

// ---- Callback function pointer type aliases ---------------------------------

/// RSA-PSS-SHA256 sign: sign digest (32 bytes) with caller's private key.
pub type FnRsaPssSign = unsafe extern "C" fn(
    digest: *const u8,
    digest_len: c_uint,
    signature: *mut u8,
    sig_len: *mut c_uint,
    user_ctx: *mut c_void,
) -> c_int;

/// RSA-OAEP-SHA256 encrypt plaintext using public key from cert_der.
pub type FnRsaOaepEncryptForCert = unsafe extern "C" fn(
    plaintext: *const u8,
    plaintext_len: c_uint,
    cert_der: *const u8,
    cert_der_len: c_uint,
    ciphertext_out: *mut u8,
    ciphertext_len: *mut c_uint,
    user_ctx: *mut c_void,
) -> c_int;

/// RSA-OAEP-SHA256 decrypt ciphertext with caller's private key.
pub type FnRsaOaepDecrypt = unsafe extern "C" fn(
    ciphertext: *const u8,
    ciphertext_len: c_uint,
    plaintext_out: *mut u8,
    plaintext_len: *mut c_uint,
    user_ctx: *mut c_void,
) -> c_int;

/// RSA-PSS-SHA256 verify signature against digest using sender's cert public key.
pub type FnRsaPssVerify = unsafe extern "C" fn(
    digest: *const u8,
    digest_len: c_uint,
    sig: *const u8,
    sig_len: c_uint,
    sender_cert_der: *const u8,
    sender_cert_der_len: c_uint,
    user_ctx: *mut c_void,
) -> c_int;

/// Progress callback: called after each (file_idx, recip_idx) pair completes.
pub type FnProgressCallback = unsafe extern "C" fn(
    file_idx: u32,
    recip_idx: u32,
    file_total: u32,
    recip_total: u32,
    status: c_int,
    user_ctx: *mut c_void,
) -> c_int;

// ---- DLL export function pointer types --------------------------------------

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

pub type FnGetError = unsafe extern "C" fn() -> c_int;

// ---- repr(C) structs matching htqt-api.h exactly ----------------------------

/// Caller-populated callbacks + context passed to enc/dec DLL functions.
#[repr(C)]
pub struct CryptoCallbacksV2 {
    pub sign_fn: Option<FnRsaPssSign>,
    pub rsa_enc_cert_fn: Option<FnRsaOaepEncryptForCert>,
    pub rsa_dec_fn: Option<FnRsaOaepDecrypt>,
    pub verify_fn: Option<FnRsaPssVerify>,
    pub progress_fn: Option<FnProgressCallback>,
    pub user_ctx: *mut c_void,
    pub own_cert_der: *const u8,        // for SF v1 compat; NULL if not available
    pub own_cert_der_len: c_uint,       // 0 = skip SF v1 fingerprint matching
    pub reserved: [*mut c_void; 3],     // must be NULL
}

// SAFETY: CryptoCallbacksV2 is used only within spawn_blocking;
// raw pointers are valid for the DLL call duration.
unsafe impl Send for CryptoCallbacksV2 {}
unsafe impl Sync for CryptoCallbacksV2 {}

/// Single plaintext file entry for batch encrypt.
#[repr(C)]
pub struct FileEntry {
    pub input_path: *const c_char, // UTF-8 path to plaintext file
    pub file_id: *const c_char,    // used in output filename: {file_id}-{recipient_id}.sf
}

unsafe impl Send for FileEntry {}
unsafe impl Sync for FileEntry {}

/// Recipient certificate entry for batch encrypt.
#[repr(C)]
pub struct RecipientEntry {
    pub cert_path: *const c_char,    // UTF-8 path to recipient DER/PEM cert
    pub recipient_id: *const c_char, // stored in SF v2 RecipientBlock + output filename
}

unsafe impl Send for RecipientEntry {}
unsafe impl Sync for RecipientEntry {}

/// Batch encrypt parameters: M files × N recipients.
#[repr(C)]
pub struct BatchEncryptParams {
    pub files: *const FileEntry,
    pub file_count: u32,
    pub recipients: *const RecipientEntry,
    pub recipient_count: u32,
    pub output_dir: *const c_char, // UTF-8 path to output directory
    pub flags: u32,
    pub reserved: [*mut c_void; 2], // must be NULL
}

unsafe impl Send for BatchEncryptParams {}
unsafe impl Sync for BatchEncryptParams {}

/// Batch decrypt parameters for SF v1 files.
#[repr(C)]
pub struct BatchSfDecryptParams {
    pub files: *const FileEntry,    // input_path = .sf1 file; file_id for result tracking
    pub file_count: u32,
    pub output_dir: *const c_char,  // filenames taken from orig_name in SF header
    pub flags: u32,
    pub reserved: [*mut c_void; 2], // must be NULL
}

unsafe impl Send for BatchSfDecryptParams {}
unsafe impl Sync for BatchSfDecryptParams {}

/// Result entry for one (file, recipient) pair in batch encrypt.
#[repr(C)]
pub struct BatchResult {
    pub file_index: u32,
    pub recipient_index: u32,
    pub status: c_int,           // HTQT_OK or HTQT_ERR_*
    pub output_path: [c_char; 512],
    pub error_detail: [c_char; 256],
}

impl Default for BatchResult {
    fn default() -> Self {
        // SAFETY: BatchResult is repr(C) with integer fields; zeroed = valid default.
        unsafe { std::mem::zeroed() }
    }
}

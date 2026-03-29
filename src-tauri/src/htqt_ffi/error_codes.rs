// ---- Error code constants ---------------------------------------------------

pub const HTQT_OK: i32 = 0;
pub const HTQT_WARN_PARTIAL_RECIP: i32 = 1;
/// Batch partial failure: some (file, recipient) pairs failed.
pub const HTQT_ERR_PARTIAL: i32 = 2;

// Batch processing flags
pub const HTQT_BATCH_CONTINUE_ON_ERROR: u32 = 0x01;
pub const HTQT_BATCH_OVERWRITE_OUTPUT: u32 = 0x02;

// ---- Error name / message functions -----------------------------------------

/// Map htqt.dll integer return code to its symbolic constant name.
pub fn htqt_error_name(code: i32) -> &'static str {
    match code {
        0   => "HTQT_OK",
        1   => "HTQT_WARN_PARTIAL_RECIP",
        2   => "HTQT_ERR_PARTIAL",
        -1  => "HTQT_ERR_OPEN_SRC",
        -2  => "HTQT_ERR_OPEN_DST",
        -3  => "HTQT_ERR_ALLOC",
        -4  => "HTQT_ERR_PKCS11_INIT",
        -5  => "HTQT_ERR_PKCS11_LOGIN",
        -6  => "HTQT_ERR_PKCS11_CERT",
        -7  => "HTQT_ERR_PKCS11_SIGN",
        -8  => "HTQT_ERR_PKCS11_DECRYPT",
        -9  => "HTQT_ERR_READ_CERT",
        -10 => "HTQT_ERR_READ_PUBKEY",
        -11 => "HTQT_ERR_WRITE_HEADER",
        -12 => "HTQT_ERR_CREATE_IV",
        -13 => "HTQT_ERR_CREATE_SESSKEY",
        -14 => "HTQT_ERR_WRAP_KEY",
        -15 => "HTQT_ERR_NO_RECIP",
        -16 => "HTQT_ERR_EXPAND_KEY",
        -17 => "HTQT_ERR_ENCRYPT",
        -18 => "HTQT_ERR_SIGN",
        -19 => "HTQT_ERR_WRITE_BODY",
        -20 => "HTQT_ERR_PARSE_SF2",
        -21 => "HTQT_ERR_UNWRAP_KEY",
        -22 => "HTQT_ERR_DECRYPT",
        -23 => "HTQT_ERR_SIG_INVALID",
        -24 => "HTQT_ERR_BAD_FORMAT",
        -25 => "HTQT_ERR_PKCS11_FIND",
        _   => "HTQT_ERR_UNKNOWN",
    }
}

/// Map htqt.dll integer return code to a human-readable error message.
pub fn htqt_error_message(code: i32) -> &'static str {
    match code {
        -1  => "Source file could not be opened. Verify the source file path and read permissions.",
        -2  => "Destination file could not be created. Verify the output directory exists and write permissions are granted.",
        -3  => "Memory allocation failed (malloc returned NULL). The system may be out of memory.",
        -4  => "PKCS#11 library could not be loaded or initialized. Verify pkcs11_lib_path is correct and the middleware driver is installed.",
        -5  => "Login failed. Verify the PIN is correct and the token is inserted. Check whether the token is locked after too many failed PIN attempts.",
        -6  => "No CERTIFICATE object found on the token. Ensure the token is personalized with a valid user certificate.",
        -7  => "RSA-PSS signature failed on the token. Check token connectivity and the CKA_SIGN attribute of the private key.",
        -8  => "RSA-OAEP decrypt failed on the token. Check the CKA_DECRYPT attribute. May also indicate wrapped-key corruption.",
        -9  => "Could not read a recipient certificate file from disk. Verify the certificate file path and that the file is a valid DER or PEM certificate.",
        -10 => "Failed to extract RSA public key from the certificate. The certificate may be malformed or use a non-RSA key type.",
        -11 => "I/O write error while writing the SF file header. Verify available disk space.",
        -12 => "Could not generate a random IV. Hardware entropy source is unavailable.",
        -13 => "Could not generate a random session key. Hardware entropy source is unavailable.",
        -14 => "RSA-OAEP encryption failed for a recipient. The recipient certificate public key may be too short or malformed.",
        -15 => "No recipient certificate was successfully wrapped. At least one valid recipient certificate is required.",
        -16 => "Session key invalid.",
        -17 => "I/O write error while writing ciphertext. Disk may be full.",
        -18 => "On-token signing error.",
        -19 => "Failed writing ciphertext length or signature to the output file.",
        -20 => "SF header parse error (reading field lengths or values). The source file may be truncated or corrupted.",
        -21 => "No recipient slot in the SF file matched this token's private key. This token is not an authorized recipient.",
        -22 => "Decryption of ciphertext failed.",
        -23 => "Signature verification failed. The file may have been tampered with.",
        -24 => "Bad file format. The source file is not a valid SF file.",
        -25 => "PKCS#11 object search failed.",
        _   => "Unknown error code returned by htqt.dll.",
    }
}

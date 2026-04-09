use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cryptoki::mechanism::rsa::{PkcsMgfType, PkcsPssParams};
use cryptoki::mechanism::{Mechanism, MechanismType};
use cryptoki::object::ObjectHandle;
use cryptoki::session::Session;
use sha2::{Digest, Sha256};

const SEPARATOR: &[u8] = b"||SIG||";

/// Sign payload bytes: SHA-256 digest → C_Sign(RsaPkcsPss, SHA-256, salt=32).
pub fn sign_payload(
    session: &Session,
    priv_key: ObjectHandle,
    payload_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    let digest = Sha256::digest(payload_bytes);
    // Use struct literal — same pattern as htqt_ffi/callbacks.rs
    let pss_params = PkcsPssParams {
        hash_alg: MechanismType::SHA256,
        mgf: PkcsMgfType::MGF1_SHA256,
        s_len: 32_usize.try_into().expect("32 fits in Ulong"),
    };
    let mechanism = Mechanism::RsaPkcsPss(pss_params);
    session
        .sign(&mechanism, priv_key, &digest)
        .map_err(|e| format!("PKCS#11 C_Sign failed: {}", e))
}

/// Assemble license.dat content: Base64(payload || SEPARATOR || signature).
pub fn assemble_license_dat(payload_bytes: &[u8], signature: &[u8]) -> String {
    let mut binary = Vec::with_capacity(payload_bytes.len() + SEPARATOR.len() + signature.len());
    binary.extend_from_slice(payload_bytes);
    binary.extend_from_slice(SEPARATOR);
    binary.extend_from_slice(signature);
    STANDARD.encode(&binary)
}

/// Write license file to output directory. Returns the full output path.
pub fn write_license_file(
    output_dir: &str,
    user_name: &str,
    content: &str,
) -> Result<String, String> {
    let safe_name = super::sanitize_user_name(user_name);
    let file_name = format!("{}-license.dat", safe_name);
    let path = std::path::Path::new(output_dir).join(&file_name);

    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write license file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

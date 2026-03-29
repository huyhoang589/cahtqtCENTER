use std::path::Path;

/// Sanitize a certificate CN into a safe filename (≤64 chars, alphanumeric + dash + underscore).
pub fn sanitize_cert_filename(cn: &str) -> String {
    let sanitized: String = cn
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if sanitized.is_empty() {
        "sender_cert".to_string()
    } else {
        sanitized
    }
}

/// Write DER bytes to DATA/Certs/sender/{sanitized_cn}.crt.
/// Returns the absolute path to the saved file.
pub fn export_cert_file(der: &[u8], sender_dir: &Path, cn: &str) -> Result<String, String> {
    let filename = format!("{}.crt", sanitize_cert_filename(cn));
    let dest = sender_dir.join(&filename);
    std::fs::write(&dest, der)
        .map_err(|e| format!("Failed to save certificate: {}", e))?;
    Ok(dest.to_string_lossy().to_string())
}

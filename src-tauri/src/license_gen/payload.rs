use serde::Serialize;
use sha2::{Digest, Sha256};

use super::MachineCredential;
use crate::db;

/// Compute machine fingerprint: hex(SHA-256(cpu_id:board_serial:token_serial)[0..8]) → 16 hex chars.
pub fn compute_machine_fp(cpu_id: &str, board_serial: &str, token_serial: &str) -> String {
    let raw = format!("{}:{}:{}", cpu_id, board_serial, token_serial);
    let hash = Sha256::digest(raw.as_bytes());
    hex::encode(&hash[..8])
}

/// License payload — struct fields in alphabetical order so serde serializes sorted keys by default.
/// Raw hardware IDs (cpu_id, board_serial, token_serial) are excluded; machine_fp encodes them.
#[derive(Debug, Serialize, Clone)]
pub struct LicensePayload {
    pub expires_at: Option<i64>,
    pub issued_at: i64,
    pub issued_by: String,
    pub machine_fp: String,
    pub product: String,
}

/// Build a LicensePayload from a credential + generation parameters.
pub fn build_payload(
    cred: &MachineCredential,
    expires_at: Option<i64>,
    server_serial: &str,
) -> LicensePayload {
    let machine_fp = compute_machine_fp(&cred.cpu_id, &cred.board_serial, &cred.token_serial);
    LicensePayload {
        expires_at,
        issued_at: db::now_secs(),
        issued_by: server_serial.to_string(),
        machine_fp,
        product: "CAHTQT_CLIENT".to_string(),
    }
}

/// Serialize payload to canonical JSON bytes with guaranteed sorted keys.
/// Uses serde_json::Value (BTreeMap internally) to ensure alphabetical key order
/// regardless of struct field declaration order across serde versions.
pub fn to_canonical_json(payload: &LicensePayload) -> Result<Vec<u8>, String> {
    let value = serde_json::to_value(payload)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    serde_json::to_vec(&value)
        .map_err(|e| format!("JSON encoding failed: {}", e))
}

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub vendor: String,
    pub description: String,
    pub path: String,
    pub cryptoki_version: String,
    pub library_version: String,
    pub manufacturer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub slot_id: u64,
    pub slot_description: String,
    pub manufacturer: String,
    pub hardware_version: String,
    pub firmware_version: String,
    pub token_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub slot_id: u64,
    pub label: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub firmware_version: String,
    pub pin_min_len: u64,
    pub pin_max_len: u64,
    pub pin_initialized: bool,
    pub user_pin_locked: bool,
    pub user_pin_final_try: bool,
    pub user_pin_count_low: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub object_id: String,       // hex-encoded CKA_ID
    pub label: String,           // CKA_LABEL
    pub subject_cn: String,
    pub subject_email: String,
    pub subject_org: String,
    pub subject_unit: String,
    pub issuer_cn: String,
    pub issuer_org: String,
    pub serial_number: String,
    pub valid_from: String,      // ISO 8601 "YYYY-MM-DD"
    pub valid_until: String,     // ISO 8601 "YYYY-MM-DD"
    pub is_expired: bool,
    pub is_ca: bool,
    pub key_usage: Vec<String>,
    pub fingerprint_sha1: String,
    #[serde(skip)]               // NOT sent to frontend — held in AppState cache only
    pub raw_der: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCertEntry {
    pub slot_id: u64,
    pub certificate: CertificateInfo,
}

/// PKCS#11 mechanism details for display in the Settings UI.
/// Queried from first slot only; sent to frontend via token_scan result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismDetail {
    pub name: String,            // "RSA_PKCS_OAEP" | "RSA_PKCS_PSS"
    pub pkcs_standard: String,   // "PKCS#1 v2.1"
    pub min_key_bits: u64,
    pub max_key_bits: u64,
    pub flags: Vec<String>,      // ["encrypt", "decrypt", "wrap"] or ["sign", "verify"]
    pub supported: bool,         // false if mechanism not present in mechanism_list
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenScanResult {
    pub library: LibraryInfo,
    pub slots: Vec<SlotInfo>,
    pub tokens: Vec<TokenInfo>,
    pub certificates: Vec<TokenCertEntry>,
    pub mechanisms: Vec<MechanismDetail>,  // RSA_PKCS_OAEP + RSA_PKCS_PSS details
    pub scan_time: String,       // ISO 8601 timestamp
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SenderCertExportResult {
    pub saved_path: String,
    pub display_name: String,
    pub email: String,
    pub organization: String,
    pub serial: String,
    pub valid_until: String,
}

// ---- Token Login State -------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenStatus {
    Disconnected,
    Connected,
    LoggedIn,
}

/// Holds verified token login state after a successful login_token call.
/// PIN stored as Zeroizing<String> — auto-zeroed from memory on drop.
#[derive(Debug)]
pub struct TokenLoginState {
    pub status: TokenStatus,
    pub pkcs11_lib_path: Option<String>,
    pub slot_id: Option<u32>,
    pub cert_cn: Option<String>,
    /// Path to sender cert DER file saved to DATA/Certs/sender/sender.crt on login
    pub sender_cert_path: Option<String>,
    /// None = not logged in, Some = verified PIN stored for DLL calls
    pub pin: Option<Zeroizing<String>>,
}

impl Default for TokenLoginState {
    fn default() -> Self {
        Self {
            status: TokenStatus::Disconnected,
            pkcs11_lib_path: None,
            slot_id: None,
            cert_cn: None,
            sender_cert_path: None,
            pin: None,
        }
    }
}

impl TokenLoginState {
    /// Clear login state — Zeroizing<String> Drop auto-zeroizes PIN memory.
    pub fn logout(&mut self) {
        *self = TokenLoginState::default();
    }

    /// Get PIN as &str for DLL calls. Returns None if not logged in.
    pub fn get_pin(&self) -> Option<&str> {
        self.pin.as_deref().map(|s| s.as_str())
    }
}

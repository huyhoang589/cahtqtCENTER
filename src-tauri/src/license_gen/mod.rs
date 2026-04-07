pub mod payload;
pub mod signer;

use serde::{Deserialize, Serialize};

/// Machine Credential JSON from client Registration Tool.
/// `rename_all = "camelCase"` matches Tauri's default serialization.
/// `alias` on each field allows reading snake_case JSON files from the Registration Tool.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MachineCredential {
    #[serde(alias = "token_serial")]
    pub token_serial: String,
    #[serde(alias = "cpu_id")]
    pub cpu_id: String,
    #[serde(alias = "board_serial")]
    pub board_serial: String,
    #[serde(alias = "user_name")]
    pub user_name: String,
    #[serde(alias = "registered_at")]
    pub registered_at: String,
}

/// Validate credential fields — reject empty, placeholder, or too-short values.
pub fn validate_credential(cred: &MachineCredential) -> Result<(), String> {
    let checks = [
        ("token_serial", &cred.token_serial),
        ("cpu_id", &cred.cpu_id),
        ("board_serial", &cred.board_serial),
        ("user_name", &cred.user_name),
    ];
    for (field, value) in &checks {
        let v = value.trim();
        if v.is_empty() || v.len() < 3 {
            return Err(format!("Invalid {}: value too short or empty", field));
        }
        if v == "To be filled by O.E.M." || v == "Default string" || v == "UNAVAILABLE" {
            return Err(format!("Invalid {}: placeholder value '{}'", field, v));
        }
    }

    // Validate registered_at is a valid YYYY-MM-DD date
    let date_str = cred.registered_at.trim();
    if date_str.is_empty() {
        return Err("Invalid registered_at: empty".to_string());
    }
    if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
        return Err(format!(
            "Invalid registered_at: '{}' is not a valid YYYY-MM-DD date",
            date_str
        ));
    }

    Ok(())
}

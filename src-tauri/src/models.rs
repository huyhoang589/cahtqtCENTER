use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Emitted as the "app_log" Tauri event for general application messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLogPayload {
    pub level: String,     // "info" | "success" | "warning" | "error"
    pub message: String,
    pub timestamp: String, // HH:MM:SS
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Partner {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

/// Partner with member count (from JOIN query)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartnerWithCount {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartnerMember {
    pub id: String,
    pub partner_id: String,
    pub name: String,
    pub email: Option<String>,
    pub cert_cn: String,
    pub cert_serial: String,
    pub cert_valid_from: i64,
    pub cert_valid_to: i64,
    pub cert_file_path: String,
    /// Organization from cert (migration 004); NULL for members added before migration
    #[sqlx(default)]
    pub cert_org: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EncLog {
    pub id: String,
    pub operation: String,
    pub src_file: String,
    pub dst_file: String,
    pub partner_member_id: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
}

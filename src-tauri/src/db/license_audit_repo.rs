use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// Row stored in the license_audit table — one entry per license generation.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LicenseAuditRow {
    pub id: String,
    pub server_serial: String,
    pub user_name: String,
    pub unit_name: String,
    pub token_serial: String,
    pub machine_fp: String,
    pub cpu_id: String,
    pub board_serial: String,
    pub product: String,
    pub expires_at: Option<i64>,
    pub output_path: String,
    pub created_at: i64,
    pub license_blob: Option<String>,
}

/// Insert a new license audit record after successful generation.
pub async fn insert_audit(
    pool: &Pool<Sqlite>,
    server_serial: &str,
    user_name: &str,
    unit_name: &str,
    token_serial: &str,
    machine_fp: &str,
    cpu_id: &str,
    board_serial: &str,
    product: &str,
    expires_at: Option<i64>,
    output_path: &str,
    license_blob: Option<&str>,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let ts = super::now_secs();

    sqlx::query(
        "INSERT INTO license_audit (id, server_serial, user_name, unit_name, token_serial, machine_fp, cpu_id, board_serial, product, expires_at, output_path, created_at, license_blob)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(server_serial)
    .bind(user_name)
    .bind(unit_name)
    .bind(token_serial)
    .bind(machine_fp)
    .bind(cpu_id)
    .bind(board_serial)
    .bind(product)
    .bind(expires_at)
    .bind(output_path)
    .bind(ts)
    .bind(license_blob)
    .execute(pool)
    .await?;
    Ok(id)
}

/// List audit records ordered by most recent first (for history table).
pub async fn list_audit(
    pool: &Pool<Sqlite>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LicenseAuditRow>, sqlx::Error> {
    sqlx::query_as::<_, LicenseAuditRow>(
        "SELECT id, server_serial, user_name, unit_name, token_serial, machine_fp, cpu_id, board_serial, product, expires_at, output_path, created_at, license_blob
         FROM license_audit ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Fetch a single audit row by id.
pub async fn get_audit_by_id(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<LicenseAuditRow>, sqlx::Error> {
    sqlx::query_as::<_, LicenseAuditRow>(
        "SELECT id, server_serial, user_name, unit_name, token_serial, machine_fp, cpu_id, board_serial, product, expires_at, output_path, created_at, license_blob
         FROM license_audit WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Hard-delete an audit record by id.
pub async fn delete_audit(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM license_audit WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

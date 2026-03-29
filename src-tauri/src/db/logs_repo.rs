use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::models::EncLog;

pub async fn insert_log(
    pool: &Pool<Sqlite>,
    operation: &str,
    src_file: &str,
    dst_file: &str,
    partner_member_id: Option<&str>,
    status: &str,
    error_msg: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let ts = super::now_secs();

    sqlx::query(
        "INSERT INTO enc_logs (id, operation, src_file, dst_file, partner_member_id, status, error_msg, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(operation)
    .bind(src_file)
    .bind(dst_file)
    .bind(partner_member_id)
    .bind(status)
    .bind(error_msg)
    .bind(ts)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_logs(
    pool: &Pool<Sqlite>,
    limit: i64,
    offset: i64,
) -> Result<Vec<EncLog>, sqlx::Error> {
    sqlx::query_as::<_, EncLog>(
        "SELECT id, operation, src_file, dst_file, partner_member_id, status, error_msg, created_at
         FROM enc_logs ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

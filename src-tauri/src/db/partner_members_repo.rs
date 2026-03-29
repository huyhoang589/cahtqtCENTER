use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::models::PartnerMember;

use super::now_secs;

#[allow(clippy::too_many_arguments)]
pub async fn add_partner_member(
    pool: &Pool<Sqlite>,
    partner_id: &str,
    name: &str,
    email: Option<&str>,
    cert_cn: &str,
    cert_serial: &str,
    cert_valid_from: i64,
    cert_valid_to: i64,
    cert_file_path: &str,
    cert_org: Option<&str>,
) -> Result<PartnerMember, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let ts = now_secs();

    sqlx::query(
        "INSERT INTO partner_members
         (id, partner_id, name, email, cert_cn, cert_serial, cert_valid_from, cert_valid_to, cert_file_path, cert_org, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(partner_id)
    .bind(name)
    .bind(email)
    .bind(cert_cn)
    .bind(cert_serial)
    .bind(cert_valid_from)
    .bind(cert_valid_to)
    .bind(cert_file_path)
    .bind(cert_org)
    .bind(ts)
    .execute(pool)
    .await?;

    Ok(PartnerMember {
        id,
        partner_id: partner_id.to_string(),
        name: name.to_string(),
        email: email.map(str::to_string),
        cert_cn: cert_cn.to_string(),
        cert_serial: cert_serial.to_string(),
        cert_valid_from,
        cert_valid_to,
        cert_file_path: cert_file_path.to_string(),
        cert_org: cert_org.map(str::to_string),
        created_at: ts,
    })
}

pub async fn list_members_by_partner(
    pool: &Pool<Sqlite>,
    partner_id: &str,
) -> Result<Vec<PartnerMember>, sqlx::Error> {
    sqlx::query_as::<_, PartnerMember>(
        "SELECT id, partner_id, name, email, cert_cn, cert_serial,
                cert_valid_from, cert_valid_to, cert_file_path, cert_org, created_at
         FROM partner_members WHERE partner_id = ? ORDER BY name ASC",
    )
    .bind(partner_id)
    .fetch_all(pool)
    .await
}

pub async fn get_partner_member(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<PartnerMember>, sqlx::Error> {
    sqlx::query_as::<_, PartnerMember>(
        "SELECT id, partner_id, name, email, cert_cn, cert_serial,
                cert_valid_from, cert_valid_to, cert_file_path, cert_org, created_at
         FROM partner_members WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_partner_member(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM partner_members WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}


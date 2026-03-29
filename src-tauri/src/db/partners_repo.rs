use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::models::{Partner, PartnerWithCount};

use super::now_secs;

pub async fn create_partner(pool: &Pool<Sqlite>, name: &str) -> Result<Partner, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let ts = now_secs();
    sqlx::query("INSERT INTO partners (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(ts)
        .execute(pool)
        .await?;
    Ok(Partner { id, name: name.to_string(), created_at: ts })
}

pub async fn list_partners(pool: &Pool<Sqlite>) -> Result<Vec<PartnerWithCount>, sqlx::Error> {
    sqlx::query_as::<_, PartnerWithCount>(
        "SELECT g.id, g.name, g.created_at, COUNT(r.id) AS member_count
         FROM partners g
         LEFT JOIN partner_members r ON r.partner_id = g.id
         GROUP BY g.id
         ORDER BY g.name ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn rename_partner(
    pool: &Pool<Sqlite>,
    id: &str,
    new_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE partners SET name = ? WHERE id = ?")
        .bind(new_name)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_partner(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    // ON DELETE CASCADE in schema handles partner_members cleanup in DB
    sqlx::query("DELETE FROM partners WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

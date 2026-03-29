use sqlx::{Pool, Sqlite};

use crate::models::Setting;

pub async fn get_setting(pool: &Pool<Sqlite>, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_as::<_, Setting>("SELECT key, value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|s| s.value))
}

pub async fn set_setting(pool: &Pool<Sqlite>, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_settings(pool: &Pool<Sqlite>) -> Result<Vec<Setting>, sqlx::Error> {
    sqlx::query_as::<_, Setting>("SELECT key, value FROM settings ORDER BY key")
        .fetch_all(pool)
        .await
}

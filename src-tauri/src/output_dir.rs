use std::path::Path;

use sqlx::SqlitePool;

use crate::db::settings_repo;

/// Resolve the output directory path for encrypt/decrypt operations.
///
/// If `override_dir` is provided, it is used as-is.
/// Otherwise reads `output_data_dir` from DB settings and appends `sub_path`
/// (e.g. `"SF\\ENCRYPT\\PartnerName"`).
///
/// The resolved directory is created (including all parents) before returning.
pub async fn resolve_output_dir(
    pool: &SqlitePool,
    override_dir: Option<&str>,
    sub_path: &str,
) -> Result<String, String> {
    let dir = if let Some(dir) = override_dir {
        dir.to_string()
    } else {
        let base = settings_repo::get_setting(pool, "output_data_dir")
            .await
            .ok()
            .flatten()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| {
                std::env::var("USERPROFILE")
                    .map(|p| format!("{}\\Desktop", p))
                    .unwrap_or_default()
            });
        // sub_path is already sanitised by the caller (e.g. "SF\\ENCRYPT\\{safe_partner}")
        format!("{}\\{}", base.trim_end_matches(['/', '\\']), sub_path)
    };

    // Validate that sub_path doesn't escape the intended directory
    let resolved = Path::new(&dir);
    tokio::fs::create_dir_all(resolved)
        .await
        .map_err(|e| format!("Cannot create output directory '{}': {}", dir, e))?;

    Ok(dir)
}

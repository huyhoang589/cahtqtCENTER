use tauri::State;

use crate::{db::logs_repo, models::EncLog, AppState};

#[tauri::command]
pub async fn list_logs(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>,
) -> Result<Vec<EncLog>, String> {
    // Cap limit to a safe range to prevent runaway queries
    let capped = limit.clamp(1, 1000);
    logs_repo::list_logs(&state.db, capped, offset)
        .await
        .map_err(|e| e.to_string())
}

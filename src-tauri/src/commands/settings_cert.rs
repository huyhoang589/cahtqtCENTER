use tauri::{AppHandle, Manager};

use crate::{
    cert_parser::{self, CertInfo},
    db::settings_repo,
    AppState,
};

/// Import sender certificate: parse, copy to DATA/Certs/sender/, persist path in settings.
#[tauri::command]
pub async fn import_sender_cert(
    cert_path: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<CertInfo, String> {
    let cert_info = cert_parser::parse_cert_file(&cert_path)
        .map_err(|e| format!("Certificate parse error: {}", e))?;

    // Copy to DATA/Certs/sender/
    let sender_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("DATA")
        .join("Certs")
        .join("sender");
    std::fs::create_dir_all(&sender_dir)
        .map_err(|e| format!("Cannot create sender cert directory: {}", e))?;

    // Use only the safe filename component (prevents path traversal)
    let filename = std::path::Path::new(&cert_path)
        .file_name()
        .ok_or("Invalid certificate path")?
        .to_string_lossy()
        .to_string();

    let dest = sender_dir.join(&filename);
    std::fs::copy(&cert_path, &dest)
        .map_err(|e| format!("Failed to copy certificate: {}", e))?;

    // Persist path in settings
    settings_repo::set_setting(&state.db, "sender_cert_path", &dest.to_string_lossy())
        .await
        .map_err(|e| e.to_string())?;

    // Return CertInfo with file_path populated
    Ok(CertInfo {
        file_path: Some(dest.to_string_lossy().to_string()),
        ..cert_info
    })
}

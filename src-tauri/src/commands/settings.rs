use std::collections::HashMap;
use std::path::Path;

use tauri::{AppHandle, Manager, State};

use crate::{
    app_log::emit_app_log,
    db::settings_repo,
    lock_helper::safe_lock,
    AppState,
};

#[derive(serde::Serialize)]
pub struct AppSettings {
    pub output_data_dir: String,
    pub pkcs11_mode: String,
    pub pkcs11_manual_path: String,
}

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub version: String,
    pub app_data_dir: String,
    pub dll_loaded: bool,
}

/// Return all settings as a key→value map
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let rows = settings_repo::get_all_settings(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|s| (s.key, s.value)).collect())
}

/// Upsert a single setting key-value pair
#[tauri::command]
pub async fn set_setting(
    app: AppHandle,
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    settings_repo::set_setting(&state.db, &key, &value)
        .await
        .map_err(|e| e.to_string())?;
    emit_app_log(&app, "info", &format!("Setting saved: {}", key));
    // A-2: auto-create output subdirs immediately when directory path is saved
    if key == "output_data_dir" && !value.is_empty() {
        if let Err(e) = create_output_subdirs(&value) {
            emit_app_log(&app, "warn", &format!("output subdir creation: {}", e));
        }
    }
    Ok(())
}

/// Return app version, data directory, and DLL load status
#[tauri::command]
pub async fn get_app_info(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppInfo, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let dll_loaded = safe_lock(&state.htqt_lib)?.is_some();
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        app_data_dir,
        dll_loaded,
    })
}

/// Return app settings: output_data_dir (falls back to Desktop), pkcs11_mode, pkcs11_manual_path
#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let all = settings_repo::get_all_settings(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let map: HashMap<String, String> = all.into_iter().map(|s| (s.key, s.value)).collect();

    let output_data_dir = map.get("output_data_dir")
        .filter(|v| !v.is_empty())
        .cloned()
        .unwrap_or_else(|| {
            std::env::var("USERPROFILE")
                .map(|p| format!("{}/Desktop", p))
                .unwrap_or_default()
        });

    Ok(AppSettings {
        output_data_dir,
        pkcs11_mode: map.get("pkcs11_mode").cloned().unwrap_or_else(|| "auto".to_string()),
        pkcs11_manual_path: map.get("pkcs11_manual_path").cloned().unwrap_or_default(),
    })
}

/// Open folder in system explorer; create it if it does not exist
#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let resolved = path.trim_end_matches(['/', '\\']).to_string();
    if resolved.is_empty() {
        return Err("Empty path provided".to_string());
    }
    std::fs::create_dir_all(&resolved)
        .map_err(|e| format!("Cannot create directory: {}", e))?;
    #[cfg(target_os = "windows")]
    {
        // Normalize to backslashes — explorer.exe treats forward slashes as
        // command-line switches, which causes it to open the wrong directory.
        let win_path = resolved.replace('/', "\\");
        std::process::Command::new("explorer")
            .arg(&win_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    return Err("open_folder only supported on Windows".to_string());
    Ok(())
}

/// Creates SF/ENCRYPT, SF/DECRYPT, SF/CERT_EXPORT, SF/SET_COMMUNICATION under `base`.
/// Safe to call multiple times (create_dir_all is idempotent).
pub fn create_output_subdirs(base: &str) -> Result<(), String> {
    let base = base.trim_end_matches(['/', '\\']);
    for sub in &["SF/ENCRYPT", "SF/DECRYPT", "SF/CERT_EXPORT", "SF/SET_COMMUNICATION"] {
        let path = format!("{}/{}", base, sub);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Cannot create {}: {}", path, e))?;
    }
    Ok(())
}

/// Copy a partner member cert file to dest_dir; create dest_dir if needed. Returns dest path.
/// C-2: if cert_cn + cert_serial provided, uses sanitized "{CN}-{Serial}.crt" as filename.
#[tauri::command]
pub fn export_member_cert(
    cert_path: String,
    dest_dir: String,
    cert_cn: Option<String>,
    cert_serial: Option<String>,
) -> Result<String, String> {
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Cannot create export directory: {}", e))?;

    // C-2: sanitize a string for use in a filename (replace forbidden chars with _)
    fn sanitize(s: &str) -> String {
        s.chars().map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        }).collect()
    }

    let original_name = Path::new(&cert_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "cert.crt".to_string());

    let file_name = match (cert_cn.as_deref(), cert_serial.as_deref()) {
        (Some(cn), Some(ser)) => format!("{}-{}.crt", sanitize(cn), sanitize(ser)),
        (Some(cn), _)         => format!("{}.crt", sanitize(cn)),
        (_, Some(ser))        => format!("{}.crt", sanitize(ser)),
        _                     => original_name,
    };

    let dest_path = format!("{}/{}", dest_dir.trim_end_matches(['/', '\\']), file_name);
    std::fs::copy(&cert_path, &dest_path)
        .map_err(|e| format!("Failed to copy certificate: {}", e))?;
    Ok(dest_path)
}

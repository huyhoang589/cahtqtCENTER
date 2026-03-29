use std::path::Path;

use tauri::{AppHandle, Manager, State};

use crate::{
    app_log::emit_app_log,
    db::settings_repo,
    etoken::{
        certificate_exporter, library_detector, token_manager,
        models::{LibraryInfo, SenderCertExportResult},
    },
    lock_helper::safe_lock,
    AppState,
};

// ---- token_export_sender_cert ------------------------------------------------

/// Export a selected certificate from the scan cache as the sender certificate.
#[tauri::command]
pub async fn token_export_sender_cert(
    cert_object_id: String,
    slot_id: u64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SenderCertExportResult, String> {
    let cert = {
        let guard = safe_lock(&state.last_token_scan)?;
        let scan = guard
            .as_ref()
            .ok_or("No scan result available. Please scan token first.")?;
        scan.certificates
            .iter()
            .find(|e| e.slot_id == slot_id && e.certificate.object_id == cert_object_id)
            .map(|e| e.certificate.clone())
            .ok_or_else(|| "Certificate not found in scan result.".to_string())?
    };

    let sender_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("DATA")
        .join("Certs")
        .join("sender");
    std::fs::create_dir_all(&sender_dir).map_err(|e| e.to_string())?;

    let saved_path =
        certificate_exporter::export_cert_file(&cert.raw_der, &sender_dir, &cert.subject_cn)?;

    let pairs = [
        ("sender_cert_path", saved_path.clone()),
        ("sender_cn", cert.subject_cn.clone()),
        ("sender_email", cert.subject_email.clone()),
        ("sender_org", cert.subject_org.clone()),
        ("sender_serial", cert.serial_number.clone()),
        ("sender_valid_until", cert.valid_until.clone()),
    ];
    for (key, value) in &pairs {
        settings_repo::set_setting(&state.db, key, value)
            .await
            .map_err(|e| e.to_string())?;
    }

    emit_app_log(
        &app,
        "success",
        &format!("Sender certificate saved: {}", cert.subject_cn),
    );

    Ok(SenderCertExportResult {
        saved_path,
        display_name: cert.subject_cn,
        email: cert.subject_email,
        organization: cert.subject_org,
        serial: cert.serial_number,
        valid_until: cert.valid_until,
    })
}

// ---- token_set_library_path --------------------------------------------------

/// Set a custom PKCS#11 library path. Validates file exists + can be loaded.
#[tauri::command]
pub async fn token_set_library_path(
    path: String,
    state: State<'_, AppState>,
) -> Result<LibraryInfo, String> {
    if !Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["dll", "so", "dylib"].contains(&ext.as_str()) {
        return Err("File must be a .dll, .so, or .dylib library".to_string());
    }

    let path_clone = path.clone();
    let info = tokio::task::spawn_blocking(move || {
        let pkcs11 = token_manager::initialize(&path_clone)?;
        let info = library_detector::get_library_info(&pkcs11, "Custom", &path_clone)?;
        let _ = pkcs11.finalize();
        Ok::<LibraryInfo, String>(info)
    })
    .await
    .map_err(|e| e.to_string())??;

    settings_repo::set_setting(&state.db, "pkcs11_library_path", &path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(info)
}

// ---- token_clear_sender_cert -------------------------------------------------

/// Clear sender certificate from settings (does NOT delete the file from disk).
#[tauri::command]
pub async fn token_clear_sender_cert(state: State<'_, AppState>) -> Result<(), String> {
    let keys = [
        "sender_cert_path",
        "sender_cn",
        "sender_email",
        "sender_org",
        "sender_serial",
        "sender_valid_until",
    ];
    for key in &keys {
        settings_repo::set_setting(&state.db, key, "")
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

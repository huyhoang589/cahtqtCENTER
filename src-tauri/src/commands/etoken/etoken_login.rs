use cryptoki::session::UserType;
use secrecy::Secret;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use zeroize::Zeroizing;

use crate::{
    app_log::emit_app_log,
    etoken::{
        certificate_reader, token_manager,
        models::TokenStatus,
    },
    lock_helper::safe_lock,
    AppState,
};

// ---- Response types ----------------------------------------------------------

#[derive(Serialize)]
pub struct LoginTokenResult {
    pub cert_cn: String,
    pub status: String, // "logged_in"
}

#[derive(Serialize)]
pub struct TokenStatusResponse {
    pub status: String, // "disconnected" | "connected" | "logged_in"
    pub cert_cn: Option<String>,
    pub dll_found: bool,
    pub dll_required_path: String,
}

// ---- login_token -------------------------------------------------------------

/// Verify PIN via PKCS#11 C_Login, then store verified state in AppState.token_login.
/// PIN stays in Zeroizing<String> for later use by encrypt_batch/decrypt_batch.
#[tauri::command]
pub async fn login_token(
    pin: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<LoginTokenResult, String> {
    // Wrap PIN immediately to ensure it's zeroized when dropped
    let pin = Zeroizing::new(pin);

    if *safe_lock(&state.is_operation_running)? {
        return Err("Cannot login while operation is in progress".to_string());
    }

    // Read pkcs11_lib_path + slot_id from last_token_scan
    let (pkcs11_lib_path, slot_id_u64) = {
        let scan = safe_lock(&state.last_token_scan)?;
        let scan = scan.as_ref().ok_or("No token scan result — scan token first")?;
        let lib_path = scan.library.path.clone();
        let slot = scan.tokens.first().ok_or("No tokens found — scan token first")?;
        (lib_path, slot.slot_id)
    };
    let slot_id_u32 = slot_id_u64 as u32;

    let pin_clone = pin.as_str().to_string();
    let lib_path_clone = pkcs11_lib_path.clone();

    let (cert_cn, verified_lib_path, verified_slot) = tokio::task::spawn_blocking(move || {
        let pkcs11 = token_manager::initialize(&lib_path_clone)?;

        let raw_slots = pkcs11
            .get_slots_with_token()
            .map_err(|e| format!("Slot enumeration failed: {}", e))?;
        let raw_slot = raw_slots
            .get(slot_id_u32 as usize)
            .ok_or("Slot index out of range")?;

        // RW session required for C_Login
        let session = pkcs11
            .open_rw_session(*raw_slot)
            .map_err(|e| format!("Failed to open RW session: {}", e))?;

        // C_Login — CKR_USER_ALREADY_LOGGED_IN treated as success
        let auth_pin = Secret::new(pin_clone.clone());
        match session.login(UserType::User, Some(&auth_pin)) {
            Ok(()) => {}
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("CKR_USER_ALREADY_LOGGED_IN") {
                    // treat as success — token already authenticated
                } else if msg.contains("CKR_PIN_INCORRECT") {
                    return Err("Incorrect PIN (CKR_PIN_INCORRECT)".to_string());
                } else if msg.contains("CKR_PIN_LOCKED") {
                    return Err("Token locked — contact administrator".to_string());
                } else {
                    return Err(format!("Login failed: {}", msg));
                }
            }
        }

        // Read cert CN for display (uses slot_id as index, same as scan)
        let certs = certificate_reader::read_all_certificates(&session, slot_id_u32 as u64)
            .unwrap_or_default();
        let cert_cn = certs.first().map(|c| c.subject_cn.clone()).unwrap_or_default();

        let _ = session.logout();
        drop(session); // close session before C_Finalize — PKCS#11 spec requires no open sessions
        let _ = pkcs11.finalize();

        Ok::<(String, String, u32), String>((cert_cn, lib_path_clone, slot_id_u32))
    })
    .await
    .map_err(|e| e.to_string())??;

    // Save sender cert DER to DATA/Certs/sender/sender.crt (always overwrite)
    let sender_cert_path = {
        let scan = safe_lock(&state.last_token_scan)?;
        if let Some(der) = scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
        {
            let sender_dir = app
                .path()
                .app_data_dir()
                .ok()
                .map(|p| p.join("DATA").join("Certs").join("sender"));
            if let Some(dir) = sender_dir {
                let _ = std::fs::create_dir_all(&dir);
                let dest = dir.join("sender.crt");
                if std::fs::write(&dest, &der).is_ok() {
                    Some(dest.to_string_lossy().to_string())
                } else {
                    None
                }
            } else { None }
        } else { None }
    };

    // Store verified state in AppState
    {
        let mut login = safe_lock(&state.token_login)?;
        login.status = TokenStatus::LoggedIn;
        login.pkcs11_lib_path = Some(verified_lib_path);
        login.slot_id = Some(verified_slot);
        // cert identity set by token_export_sender_cert — preserve if already set
        if login.cert_cn.is_none() {
            login.cert_cn = Some(cert_cn.clone());
        }
        if login.sender_cert_path.is_none() {
            login.sender_cert_path = sender_cert_path.clone();
        }
        login.pin = Some(pin); // Zeroizing<String> — stored until logout or app restart
    }

    if let Some(ref p) = sender_cert_path {
        emit_app_log(&app, "info", &format!("Sender cert saved: {}", p));
    }
    emit_app_log(&app, "success", &format!("Token authenticated: {}", cert_cn));
    Ok(LoginTokenResult { cert_cn, status: "logged_in".to_string() })
}

// ---- logout_token ------------------------------------------------------------

/// Clear login state — zeroizes PIN from memory, resets to Disconnected.
#[tauri::command]
pub async fn logout_token(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    safe_lock(&state.token_login)?.logout();
    emit_app_log(&app, "info", "Token logged out");
    Ok(())
}

// ---- get_token_status --------------------------------------------------------

/// Poll current token status for UI. Returns status, cert_cn, and dll_found flag.
#[tauri::command]
pub async fn get_token_status(state: State<'_, AppState>) -> Result<TokenStatusResponse, String> {
    let dll_found = safe_lock(&state.htqt_lib)?.is_some();
    let login = safe_lock(&state.token_login)?;
    let status = match login.status {
        TokenStatus::Disconnected => "disconnected",
        TokenStatus::Connected => "connected",
        TokenStatus::LoggedIn => "logged_in",
    };
    Ok(TokenStatusResponse {
        status: status.to_string(),
        cert_cn: login.cert_cn.clone(),
        dll_found,
        dll_required_path: state.dll_required_path.clone(),
    })
}

use std::ffi::{CString, c_void};
use std::path::Path;
use std::ptr;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{
    app_log::emit_app_log,
    db::logs_repo,
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks,
        error_codes::{HTQT_BATCH_CONTINUE_ON_ERROR, HTQT_BATCH_OVERWRITE_OUTPUT},
        token_context::open_token_session,
        CryptoCallbacksV2,
    },
    lock_helper::{safe_lock, OperationGuard},
    AppState,
};

#[derive(Serialize, Clone)]
pub struct DecryptProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
    pub file_path: String,
    pub status: String, // "processing" | "success" | "error"
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct DecryptResult {
    pub total: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Decrypt M .sf files using decrypt_one_sfv1 with callback-based crypto.
/// One PKCS#11 session opened; N per-file decrypt_one_sfv1 calls inside spawn_blocking.
#[tauri::command]
pub async fn decrypt_batch(
    app: AppHandle,
    file_paths: Vec<String>,
    partner_name: String,
    output_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<DecryptResult, String> {
    let _guard = OperationGuard::acquire(&state.is_operation_running)?;
    run_decrypt_batch(&app, &file_paths, &partner_name, output_dir.as_deref(), &state).await
}

async fn run_decrypt_batch(
    app: &AppHandle,
    file_paths: &[String],
    partner_name: &str,
    output_dir_override: Option<&str>,
    state: &State<'_, AppState>,
) -> Result<DecryptResult, String> {
    // Read and validate token login state; cert_cn for log only (fingerprint via own_cert_der)
    let (pkcs11_lib, slot_id, pin_str, cert_cn_log) = {
        let login = safe_lock(&state.token_login)?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in — login via Settings first".to_string());
        }
        let pin = login.get_pin().ok_or("PIN not available — re-login required")?.to_string();
        let cert_cn = login.cert_cn.clone().unwrap_or_default(); // for log only
        (
            login.pkcs11_lib_path.clone().unwrap_or_default(),
            login.slot_id.unwrap_or(0),
            pin,
            cert_cn,
        )
    };

    // Get sender's own cert DER (for SF v1 backward compat)
    let own_cert_der: Vec<u8> = {
        let scan = safe_lock(&state.last_token_scan)?;
        scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
            .unwrap_or_default()
    };

    // Resolve output directory: use override if provided, else output_data_dir\SF\DECRYPT\{partner}
    let safe_name = Path::new(partner_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let output_dir_str = crate::output_dir::resolve_output_dir(
        &state.db,
        output_dir_override,
        &format!("SF\\DECRYPT\\{}", safe_name),
    )
    .await?;

    let total = file_paths.len();
    emit_app_log(app, "info", &format!("Starting decryption: {} file(s) (recipient: {})", total, cert_cn_log));

    let file_count = file_paths.len();
    let htqt_lib_arc = state.htqt_lib.clone();
    let app_clone = app.clone();
    let own_cert_der_clone = own_cert_der.clone();
    let file_paths_owned = file_paths.to_vec();
    let output_dir_str_clone = output_dir_str.clone();

    // Per-file decrypt: one session, N × decrypt_one_sfv1 calls
    let dec_results = tokio::task::spawn_blocking(move || -> Result<Vec<(usize, Result<String, String>)>, String> {
        let output_dir_cstring = CString::new(output_dir_str_clone.as_str())
            .map_err(|e| e.to_string())?;
        let input_cstrings: Vec<CString> = file_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;

        // Open ONE PKCS#11 session for all decrypt calls
        let ctx = open_token_session(
            &pkcs11_lib,
            slot_id,
            &pin_str,
            app_clone,
            own_cert_der_clone.clone(),
            "decrypt-progress".to_string(),
        )?;
        let ctx_box = Box::new(ctx);
        let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

        let cbs = CryptoCallbacksV2 {
            sign_fn: None,
            rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
            progress_fn: None,
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der_clone.is_empty() { ptr::null() } else { own_cert_der_clone.as_ptr() },
            own_cert_der_len: own_cert_der_clone.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        // Loop: one decrypt_one_sfv1 call per file (DLL_LOCK acquired/released per call)
        let guard = crate::lock_helper::safe_lock(&htqt_lib_arc)?;
        let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;

        let mut file_results: Vec<(usize, Result<String, String>)> = Vec::with_capacity(file_count);
        for (i, cstr) in input_cstrings.iter().enumerate() {
            let result = lib.decrypt_one_sfv1(
                cstr.as_ptr(),
                output_dir_cstring.as_ptr(),
                &cbs,
                HTQT_BATCH_CONTINUE_ON_ERROR | HTQT_BATCH_OVERWRITE_OUTPUT,
            );
            file_results.push((i, result));
        }
        drop(guard);
        drop(ctx_box); // closes session

        Ok(file_results)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Emit progress events and log to DB
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for (i, (file_idx, result)) in dec_results.iter().enumerate() {
        let file_path = file_paths.get(*file_idx).map(String::as_str).unwrap_or("?");

        let file_name = Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string());

        let (status_str, error_msg, dst_for_log) = match result {
            Ok(output_path) => {
                success_count += 1;
                let dst = if output_path.is_empty() { &output_dir_str } else { output_path };
                ("success".to_string(), None, dst.clone())
            }
            Err(err_str) => {
                error_count += 1;
                emit_app_log(app, "error", &format!("[DECRYPT] {}: {}", file_name, err_str));
                errors.push(format!("{}: {}", file_name, err_str));
                ("error".to_string(), Some(err_str.clone()), output_dir_str.clone())
            }
        };

        // Emit per-file progress event
        let _ = app.emit("decrypt-progress", DecryptProgress {
            current: i + 1,
            total,
            file_name: file_name.clone(),
            file_path: file_path.to_string(),
            status: status_str.clone(),
            error: error_msg.clone(),
        });

        // Log to database
        let _ = logs_repo::insert_log(
            &state.db,
            "DECRYPT",
            file_path,
            &dst_for_log,
            None,
            &status_str,
            error_msg.as_deref(),
        )
        .await;
    }

    emit_app_log(
        app,
        if error_count == 0 { "success" } else { "warning" },
        &format!("Decryption complete: {}/{} succeeded", success_count, total),
    );
    Ok(DecryptResult { total, success_count, error_count, errors })
}

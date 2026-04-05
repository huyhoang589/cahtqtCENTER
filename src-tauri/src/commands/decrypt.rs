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
        error_codes::{HTQT_BATCH_CONTINUE_ON_ERROR, HTQT_OK},
        htqt_error_message, htqt_error_name,
        token_context::open_token_session,
        BatchResult, BatchSfDecryptParams, CryptoCallbacksV2, FileEntry,
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

/// Decrypt M .sf files using decHTQT_v2 with callback-based crypto.
/// recipient_id sourced from AppState.token_login.cert_cn (no frontend param needed).
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

    // Batch decrypt: build BatchSfDecryptParams, single dec_sf() call
    let dec_results = tokio::task::spawn_blocking(move || -> Result<Vec<BatchResult>, String> {
        let input_cstrings: Vec<CString> = file_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        // file_id = stem for result tracking
        let file_id_cstrings: Vec<CString> = file_paths_owned.iter()
            .map(|p| {
                let stem = Path::new(p).file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string());
                CString::new(stem).map_err(|e| e.to_string())
            })
            .collect::<Result<_, _>>()?;
        let output_dir_cstring = CString::new(output_dir_str_clone.as_str())
            .map_err(|e| e.to_string())?;

        let file_entries: Vec<FileEntry> = (0..file_count)
            .map(|i| FileEntry {
                input_path: input_cstrings[i].as_ptr(),
                file_id: file_id_cstrings[i].as_ptr(),
            })
            .collect();

        let params = BatchSfDecryptParams {
            files: file_entries.as_ptr(),
            file_count: file_count as u32,
            output_dir: output_dir_cstring.as_ptr(),
            flags: HTQT_BATCH_CONTINUE_ON_ERROR,
            reserved: [ptr::null_mut(); 2],
        };

        let mut batch_results: Vec<BatchResult> = (0..file_count)
            .map(|_| BatchResult::default())
            .collect();

        // Open PKCS#11 session for decrypt operations
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

        // For decrypt: sign_fn + rsa_enc_cert_fn are NOT required (null/None)
        let cbs = CryptoCallbacksV2 {
            sign_fn: None,
            rsa_enc_cert_fn: None,
            rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
            verify_fn: Some(callbacks::cb_rsa_pss_verify),
            progress_fn: None, // decrypt does not use progress callback
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der_clone.is_empty() { ptr::null() } else { own_cert_der_clone.as_ptr() },
            own_cert_der_len: own_cert_der_clone.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        let guard = crate::lock_helper::safe_lock(&htqt_lib_arc)?;
        let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;
        lib.dec_sf(&params, &cbs, &mut batch_results)?;
        drop(guard);
        drop(ctx_box); // closes session + finalizes Pkcs11

        Ok(batch_results)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Emit progress events and log to DB
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for (i, result) in dec_results.iter().enumerate() {
        let fi = result.file_index as usize;
        let file_path = file_paths.get(fi).map(String::as_str).unwrap_or("?");
        // output_path filled by DLL from SF header orig_name
        let output_path = crate::ffi_helpers::string_from_c_buf(&result.output_path);
        let dst_for_log = if output_path.is_empty() { &output_dir_str } else { &output_path };

        let file_name = Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string());

        let (status_str, error_msg) = if result.status == HTQT_OK {
            success_count += 1;
            ("success".to_string(), None)
        } else {
            error_count += 1;
            let name = htqt_error_name(result.status);
            let message = htqt_error_message(result.status);
            let detail = crate::ffi_helpers::string_from_c_buf(&result.error_detail);
            let error_str = if detail.is_empty() {
                format!("[{}] {}: {}", result.status, name, message)
            } else {
                format!("[{}] {}: {} — {}", result.status, name, message, detail)
            };
            emit_app_log(app, "error", &format!("[DECRYPT] {}: {}", file_name, error_str));
            errors.push(format!("{}: {}", file_name, error_str));
            ("error".to_string(), Some(error_str))
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
            dst_for_log,
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

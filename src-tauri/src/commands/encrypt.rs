use std::ffi::{CString, c_void};
use std::path::Path;
use std::ptr;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use tauri::Emitter;

use crate::{
    app_log::emit_app_log,
    cert_parser,
    db::logs_repo,
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks,
        error_codes::{HTQT_BATCH_CONTINUE_ON_ERROR, HTQT_BATCH_OVERWRITE_OUTPUT},
        htqt_error_message, htqt_error_name,
        token_context::open_token_session, BatchEncryptParams, BatchResult, CryptoCallbacksV2,
        FileEntry, RecipientEntry, HTQT_OK,
    },
    lock_helper::{safe_lock, OperationGuard},
    AppState,
};

#[derive(Serialize, Clone)]
pub struct EncryptProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
    pub file_path: String,
    pub status: String, // "processing" | "success" | "warning" | "error"
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct EncryptRequest {
    pub src_paths: Vec<String>,
    pub partner_name: String,
    pub cert_paths: Vec<String>,
}

#[derive(Serialize)]
pub struct EncryptResult {
    pub total: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Batch encrypt M files × N recipients via single encHTQT_multi DLL call.
/// Progress emitted per (file, recipient) pair via cb_progress callback.
#[tauri::command]
pub async fn encrypt_batch(
    app: AppHandle,
    src_paths: Vec<String>,
    partner_name: String,
    cert_paths: Vec<String>,
    output_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<EncryptResult, String> {
    let _guard = OperationGuard::acquire(&state.is_operation_running)?;
    run_encrypt_batch(&app, &src_paths, &partner_name, &cert_paths, output_dir.as_deref(), &state).await
}

async fn run_encrypt_batch(
    app: &AppHandle,
    src_paths: &[String],
    partner_name: &str,
    cert_paths: &[String],
    output_dir_override: Option<&str>,
    state: &State<'_, AppState>,
) -> Result<EncryptResult, String> {
    // Read and validate token login state
    let (pkcs11_lib, slot_id, pin_str) = {
        let login = safe_lock(&state.token_login)?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in — login via Settings first".to_string());
        }
        let pin = login.get_pin().ok_or("PIN not available — re-login required")?.to_string();
        (
            login.pkcs11_lib_path.clone().unwrap_or_default(),
            login.slot_id.unwrap_or(0),
            pin,
        )
    };

    if cert_paths.is_empty() {
        return Err("No recipient certificates provided".to_string());
    }

    // Get sender's own cert DER (first cert from last scan, for SF v1 backward compat)
    let own_cert_der: Vec<u8> = {
        let scan = safe_lock(&state.last_token_scan)?;
        scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
            .unwrap_or_default()
    };

    // Resolve output directory: use override if provided, else output_data_dir\SF\ENCRYPT\{partner}
    let safe_partner = Path::new(partner_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let output_dir_string = crate::output_dir::resolve_output_dir(
        &state.db,
        output_dir_override,
        &format!("SF\\ENCRYPT\\{}", safe_partner),
    )
    .await?;

    let file_count = src_paths.len();
    let recip_count = cert_paths.len();

    emit_app_log(app, "info",
        &format!("Starting encryption: {} file(s) × {} recipient(s)", file_count, recip_count));

    let output_dir_str = output_dir_string.clone();
    let date_suffix = chrono::Local::now().format("_%d%m%Y").to_string();

    // Pre-compute file_ids ({stem}_{DDMMYYYY}) — DLL uses this in output filename
    let file_id_strings: Vec<String> = src_paths
        .iter()
        .map(|p| {
            let stem = Path::new(p)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());
            format!("{}{}", stem, date_suffix)
        })
        .collect();

    // Extract recipient_id (cert CN) from each cert file via cert_parser
    let recipient_id_strings: Vec<String> = cert_paths
        .iter()
        .map(|cp| {
            cert_parser::parse_cert_file(cp)
                .map(|info| info.cn)
                .unwrap_or_else(|_| {
                    Path::new(cp)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "recipient".to_string())
                })
        })
        .collect();

    // Clone data for spawn_blocking (no refs across await)
    let src_paths_owned = src_paths.to_vec();
    let cert_paths_owned = cert_paths.to_vec();
    let htqt_lib_arc = state.htqt_lib.clone();
    let app_clone = app.clone();
    let recip_ids_clone = recipient_id_strings.clone();
    let file_ids_clone = file_id_strings.clone();

    let batch_results = tokio::task::spawn_blocking(move || -> Result<Vec<BatchResult>, String> {
        // Build CString arrays — all must outlive the DLL call
        let input_cstrings: Vec<CString> = src_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let file_id_cstrings: Vec<CString> = file_ids_clone.iter()
            .map(|s| CString::new(s.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let cert_path_cstrings: Vec<CString> = cert_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let recip_id_cstrings: Vec<CString> = recip_ids_clone.iter()
            .map(|s| CString::new(s.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let output_dir_cstring = CString::new(output_dir_str).map_err(|e| e.to_string())?;

        let file_entries: Vec<FileEntry> = (0..file_count)
            .map(|i| FileEntry {
                input_path: input_cstrings[i].as_ptr(),
                file_id: file_id_cstrings[i].as_ptr(),
            })
            .collect();

        let recip_entries: Vec<RecipientEntry> = (0..recip_count)
            .map(|i| RecipientEntry {
                cert_path: cert_path_cstrings[i].as_ptr(),
                recipient_id: recip_id_cstrings[i].as_ptr(),
            })
            .collect();

        let params = BatchEncryptParams {
            files: file_entries.as_ptr(),
            file_count: file_count as u32,
            recipients: recip_entries.as_ptr(),
            recipient_count: recip_count as u32,
            output_dir: output_dir_cstring.as_ptr(),
            flags: HTQT_BATCH_CONTINUE_ON_ERROR | HTQT_BATCH_OVERWRITE_OUTPUT,
            reserved: [ptr::null_mut(); 2],
        };

        // Open PKCS#11 session for this batch operation
        let ctx = open_token_session(
            &pkcs11_lib,
            slot_id,
            &pin_str,
            app_clone,
            own_cert_der.clone(),
            "encrypt-progress".to_string(),
        )?;

        let ctx_box = Box::new(ctx);
        let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

        let cbs = CryptoCallbacksV2 {
            sign_fn: Some(callbacks::cb_rsa_pss_sign),
            rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
            progress_fn: Some(callbacks::cb_progress),
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der.is_empty() { ptr::null() } else { own_cert_der.as_ptr() },
            own_cert_der_len: own_cert_der.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        let mut batch_results: Vec<BatchResult> = (0..file_count)
            .map(|_| BatchResult::default())
            .collect();

        let guard = crate::lock_helper::safe_lock(&htqt_lib_arc)?;
        match guard.as_ref() {
            None => return Err("htqt_crypto.dll not loaded".to_string()),
            Some(lib) => { lib.enc_multi(&params, &cbs, &mut batch_results)?; }
        }
        drop(guard);
        drop(ctx_box); // closes PKCS#11 session + finalizes Pkcs11

        Ok(batch_results)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Collect results per file, emit progress events, and log to DB
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for result in batch_results.iter() {
        let fi = result.file_index as usize;
        let file_path_str = src_paths.get(fi).map(String::as_str).unwrap_or("?");
        let output_path = crate::ffi_helpers::string_from_c_buf(&result.output_path);

        let file_name = Path::new(file_path_str)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path_str.to_string());

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
            errors.push(format!("{}: {}", file_name, error_str));
            ("error".to_string(), Some(error_str))
        };

        // Emit per-file progress event for UI status tracking
        let _ = app.emit("encrypt-progress", EncryptProgress {
            current: fi + 1,
            total: file_count,
            file_name: file_name.clone(),
            file_path: file_path_str.to_string(),
            status: status_str.clone(),
            error: error_msg.clone(),
        });

        let _ = logs_repo::insert_log(
            &state.db,
            "ENCRYPT",
            file_path_str,
            &output_path,
            None,
            &status_str,
            error_msg.as_deref(),
        )
        .await;
    }

    let total = batch_results.len();
    emit_app_log(
        app,
        if error_count == 0 { "success" } else { "warning" },
        &format!("Encryption complete: {}/{} succeeded", success_count, total),
    );
    Ok(EncryptResult { total, success_count, error_count, errors })
}

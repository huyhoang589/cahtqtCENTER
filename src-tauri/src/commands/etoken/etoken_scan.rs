use cryptoki::mechanism::MechanismType;
use tauri::{AppHandle, State};

use crate::{
    app_log::emit_app_log,
    db::settings_repo,
    etoken::{
        certificate_reader, library_detector, token_manager,
        models::{
            LibraryInfo, MechanismDetail, SlotInfo, TokenCertEntry, TokenScanResult, TokenStatus,
        },
    },
    lock_helper::safe_lock,
    AppState,
};

// ---- token_scan --------------------------------------------------------------

/// Full token scan: auto-detect or use override path, enumerate slots+tokens, read all certs.
/// Result is cached in AppState.last_token_scan.
/// Always resets token_login: Connected if tokens found, Disconnected otherwise.
#[tauri::command]
pub async fn token_scan(
    lib_path_override: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TokenScanResult, String> {
    {
        let running = safe_lock(&state.is_operation_running)?;
        if *running {
            return Err("Cannot scan token while encryption/decryption is in progress".to_string());
        }
    }

    // Step 1: DLL check — verify htqt.dll exists and loads cleanly
    let dll_required_path = state.dll_required_path.clone();
    if !dll_required_path.is_empty() {
        let p = std::path::Path::new(&dll_required_path);
        if p.exists() {
            match crate::htqt_ffi::HtqtLib::load(&dll_required_path) {
                Ok(lib) => {
                    *safe_lock(&state.htqt_lib)? = Some(lib);
                    emit_app_log(&app, "success",
                        &format!("htqt.dll loaded: {}", dll_required_path));
                }
                Err(e) => {
                    emit_app_log(&app, "error",
                        &format!("htqt.dll found but failed to load: {}", e));
                }
            }
        } else {
            emit_app_log(&app, "error",
                &format!("required lib path: {}", dll_required_path));
        }
    }

    // Clear previous cache
    *safe_lock(&state.last_token_scan)? = None;

    // Load saved library path from settings
    let custom_path = settings_repo::get_all_settings(&state.db)
        .await
        .ok()
        .and_then(|settings| {
            settings
                .into_iter()
                .find(|s| s.key == "pkcs11_library_path")
        })
        .map(|s| s.value);

    let resolved_path = lib_path_override.or(custom_path);

    let result = tokio::task::spawn_blocking(move || run_full_scan(resolved_path.as_deref()))
        .await
        .map_err(|e| e.to_string())??;

    emit_app_log(
        &app,
        "info",
        &format!("Token scan complete: {} cert(s) found", result.certificates.len()),
    );
    emit_app_log(
        &app,
        "info",
        &format!("pkcs11_lib_path: {}", result.library.path),
    );
    if let Some(token) = result.tokens.first() {
        emit_app_log(
            &app,
            "info",
            &format!("token slot ID: {}", token.slot_id),
        );
    }
    // Emit mechanism support lines with key size info
    for m in &result.mechanisms {
        let (mark, level) = if m.supported { ("✓", "info") } else { ("✗ MISSING", "warning") };
        emit_app_log(&app, level, &format!("  {} {} ({}–{} bits)", mark, m.name, m.min_key_bits, m.max_key_bits));
    }

    // Cache result in AppState
    *safe_lock(&state.last_token_scan)? = Some(result.clone());

    // Reset token_login state based on scan result (always force re-login after scan)
    {
        let mut login = safe_lock(&state.token_login)?;
        if !result.tokens.is_empty() {
            // Connected: scan succeeded + token present — user must re-login
            login.status = TokenStatus::Connected;
            login.pkcs11_lib_path = Some(result.library.path.clone());
            login.slot_id = result.tokens.first().map(|t| t.slot_id as u32);
            login.cert_cn = None;
            login.pin = None; // Force re-login even if previously LoggedIn
        } else {
            // No tokens found — reset to Disconnected
            *login = crate::etoken::models::TokenLoginState::default();
        }
    }

    Ok(result)
}

/// Blocking inner scan — runs all PKCS#11 operations synchronously.
fn run_full_scan(user_path: Option<&str>) -> Result<TokenScanResult, String> {
    let candidate = library_detector::auto_detect_library(user_path).ok_or_else(|| {
        "No eToken middleware detected. Please install bit4ID Universal Middleware and try again."
            .to_string()
    })?;

    let pkcs11 = token_manager::initialize(&candidate.path)?;
    let lib_info = library_detector::get_library_info(&pkcs11, &candidate.vendor, &candidate.path)?;

    let (slots, raw_slots) = token_manager::get_all_slots(&pkcs11)?;
    let tokens = token_manager::get_token_infos(&pkcs11, &slots, &raw_slots);

    // Query mechanism details from first slot (app requires OAEP + PSS)
    let mechanisms: Vec<MechanismDetail> = if let Some(&raw_slot) = raw_slots.first() {
        let supported_list = pkcs11.get_mechanism_list(raw_slot).unwrap_or_default();
        let targets = [
            (MechanismType::RSA_PKCS_OAEP, "RSA_PKCS_OAEP", "PKCS#1 v2.1"),
            (MechanismType::RSA_PKCS_PSS,  "RSA_PKCS_PSS",  "PKCS#1 v2.1"),
        ];
        targets.iter().map(|(mech_type, name, standard)| {
            if !supported_list.contains(mech_type) {
                return MechanismDetail {
                    name: name.to_string(),
                    pkcs_standard: standard.to_string(),
                    min_key_bits: 0,
                    max_key_bits: 0,
                    flags: vec![],
                    supported: false,
                };
            }
            match pkcs11.get_mechanism_info(raw_slot, *mech_type) {
                Ok(info) => {
                    let mut flags = Vec::new();
                    if info.encrypt()  { flags.push("encrypt".into()); }
                    if info.decrypt()  { flags.push("decrypt".into()); }
                    if info.sign()     { flags.push("sign".into()); }
                    if info.verify()   { flags.push("verify".into()); }
                    if info.wrap()     { flags.push("wrap".into()); }
                    if info.unwrap()   { flags.push("unwrap".into()); }
                    MechanismDetail {
                        name: name.to_string(),
                        pkcs_standard: standard.to_string(),
                        min_key_bits: info.min_key_size() as u64,
                        max_key_bits: info.max_key_size() as u64,
                        flags,
                        supported: true,
                    }
                }
                Err(_) => MechanismDetail {
                    name: name.to_string(),
                    pkcs_standard: standard.to_string(),
                    min_key_bits: 0,
                    max_key_bits: 0,
                    flags: vec![],
                    supported: false,
                },
            }
        }).collect()
    } else {
        vec![]
    };

    let cert_entries = collect_cert_entries(&pkcs11, &slots, &raw_slots);

    let _ = pkcs11.finalize();

    Ok(TokenScanResult {
        library: lib_info,
        slots,
        tokens,
        certificates: cert_entries,
        mechanisms,
        scan_time: chrono::Utc::now().to_rfc3339(),
        error: None,
    })
}

/// Read all certificates from slots that have a token present.
fn collect_cert_entries(
    pkcs11: &cryptoki::context::Pkcs11,
    slots: &[SlotInfo],
    raw_slots: &[cryptoki::slot::Slot],
) -> Vec<TokenCertEntry> {
    let mut cert_entries = Vec::new();
    for (i, slot_info) in slots.iter().enumerate().filter(|(_, s)| s.token_present) {
        let raw_slot = raw_slots[i];
        match pkcs11.open_ro_session(raw_slot) {
            Ok(session) => {
                let certs = certificate_reader::read_all_certificates(&session, slot_info.slot_id)
                    .unwrap_or_default();
                for cert in certs {
                    cert_entries.push(TokenCertEntry {
                        slot_id: slot_info.slot_id,
                        certificate: cert,
                    });
                }
            }
            Err(_) => continue,
        }
    }
    cert_entries
}

// ---- token_get_library_info --------------------------------------------------

/// Quick library detection without full scan — used on Settings page load.
#[tauri::command]
pub async fn token_get_library_info(
    state: State<'_, AppState>,
) -> Result<LibraryInfo, String> {
    let custom_path = settings_repo::get_all_settings(&state.db)
        .await
        .ok()
        .and_then(|s| s.into_iter().find(|kv| kv.key == "pkcs11_library_path"))
        .map(|kv| kv.value);

    tokio::task::spawn_blocking(move || {
        let candidate = library_detector::auto_detect_library(custom_path.as_deref())
            .ok_or_else(|| "No eToken middleware detected.".to_string())?;
        let pkcs11 = token_manager::initialize(&candidate.path)?;
        let info = library_detector::get_library_info(&pkcs11, &candidate.vendor, &candidate.path)?;
        let _ = pkcs11.finalize();
        Ok::<LibraryInfo, String>(info)
    })
    .await
    .map_err(|e| e.to_string())?
}

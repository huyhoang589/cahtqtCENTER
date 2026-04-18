use serde::Serialize;
use tauri::State;

use crate::{
    db::license_audit_repo,
    etoken::{models::TokenStatus, token_manager},
    license_gen::{
        self,
        payload::{self, LicensePayload},
        signer, MachineCredential,
    },
    lock_helper::{safe_lock, OperationGuard},
    AppState,
};

// ---- Response types --------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialPreview {
    pub credential: MachineCredential,
    pub machine_fp: String,
    pub is_valid: bool,
    pub validation_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateLicenseResult {
    pub success: bool,
    pub output_path: String,
    pub machine_fp: String,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseAuditEntry {
    pub id: String,
    pub server_serial: String,
    pub user_name: String,
    pub unit_name: String,
    pub token_serial: String,
    pub machine_fp: String,
    pub product: String,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub license_blob: Option<String>,
}

// ---- Commands --------------------------------------------------------------

/// Import and validate a Machine Credential JSON file.
#[tauri::command]
pub async fn import_credential(path: String) -> Result<CredentialPreview, String> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Cannot read file: {}", e))?;

    let cred: MachineCredential = serde_json::from_str(&contents)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let machine_fp = payload::compute_machine_fp(&cred.cpu_id, &cred.board_serial, &cred.token_serial);

    let (is_valid, validation_error) = match license_gen::validate_credential(&cred) {
        Ok(()) => (true, None),
        Err(msg) => (false, Some(msg)),
    };

    Ok(CredentialPreview {
        credential: cred,
        machine_fp,
        is_valid,
        validation_error,
    })
}

/// Generate a signed license.dat file using the server's PKCS#11 token.
#[tauri::command]
pub async fn generate_license(
    credential: MachineCredential,
    expires_at: Option<i64>,
    unit_name: String,
    state: State<'_, AppState>,
) -> Result<GenerateLicenseResult, String> {
    // Acquire operation guard — prevents concurrent token sessions
    let _guard = OperationGuard::acquire(&state.is_operation_running)?;

    // Extract token login state
    let (pkcs11_lib, slot_id, pin_str) = {
        let login = safe_lock(&state.token_login)?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in — login via Settings first".to_string());
        }
        let pin = login
            .get_pin()
            .ok_or("PIN not available — re-login required")?
            .to_string();
        (
            login.pkcs11_lib_path.clone().unwrap_or_default(),
            login.slot_id.unwrap_or(0),
            pin,
        )
    };

    // Validate credential
    license_gen::validate_credential(&credential)?;

    // Validate expires_at is in the future (if provided)
    if let Some(exp) = expires_at {
        let now = crate::db::now_secs();
        if exp <= now {
            return Err("Expiry date must be in the future".to_string());
        }
    }

    // Resolve output directory: SF\LICENSE\{User_name}
    let safe_name = license_gen::sanitize_user_name(&credential.user_name);
    let sub_path = format!("SF\\LICENSE\\{}", safe_name);
    let output_dir = crate::output_dir::resolve_output_dir(
        &state.db,
        None,
        &sub_path,
    )
    .await?;

    // Compute machine fingerprint
    let machine_fp = payload::compute_machine_fp(&credential.cpu_id, &credential.board_serial, &credential.token_serial);

    // Clone data for spawn_blocking
    let cred_clone = credential.clone();
    let unit_name_clone = unit_name.clone();
    let machine_fp_clone = machine_fp.clone();
    let output_dir_clone = output_dir.clone();

    // PKCS#11 signing in blocking thread (token I/O is synchronous)
    let (output_path, server_serial, payload_data, license_content) =
        tokio::task::spawn_blocking(move || -> Result<(String, String, LicensePayload, String), String> {
            // Initialize PKCS#11 + open session
            let pkcs11 = token_manager::initialize(&pkcs11_lib)?;
            let raw_slots = pkcs11
                .get_slots_with_token()
                .map_err(|e| format!("Slot enumeration failed: {}", e))?;
            let slot = raw_slots
                .get(slot_id as usize)
                .ok_or_else(|| format!("Slot index {} out of range", slot_id))?;

            // Read server token serial
            let token_info = pkcs11
                .get_token_info(*slot)
                .map_err(|e| format!("Failed to read token info: {}", e))?;
            let server_serial = token_info.serial_number().trim().to_string();

            let session = pkcs11
                .open_rw_session(*slot)
                .map_err(|e| format!("Failed to open RW session: {}", e))?;

            // Login
            let auth_pin = secrecy::Secret::new(pin_str);
            match session.login(cryptoki::session::UserType::User, Some(&auth_pin)) {
                Ok(()) => {}
                Err(e) => {
                    let msg = e.to_string();
                    if !msg.contains("CKR_USER_ALREADY_LOGGED_IN") {
                        return Err(format!("PKCS#11 login failed: {}", msg));
                    }
                }
            }

            // Find private signing key
            let template = vec![
                cryptoki::object::Attribute::Class(cryptoki::object::ObjectClass::PRIVATE_KEY),
                cryptoki::object::Attribute::Sign(true),
            ];
            let keys = session
                .find_objects(&template)
                .map_err(|e| format!("Failed to find private key: {}", e))?;
            let priv_key = keys
                .first()
                .ok_or("No private signing key found on token")?
                .clone();

            // Build payload + sign
            let lp = payload::build_payload(&cred_clone, expires_at, &server_serial);
            let payload_bytes = payload::to_canonical_json(&lp)?;
            let signature = signer::sign_payload(&session, priv_key, &payload_bytes)?;
            let license_content = signer::assemble_license_dat(&payload_bytes, &signature);
            let out_path =
                signer::write_license_file(&output_dir_clone, &cred_clone.user_name, &license_content)?;

            // Session + Pkcs11 dropped here (RAII)
            Ok((out_path, server_serial, lp, license_content))
        })
        .await
        .map_err(|e| e.to_string())??;

    // Insert audit record with blob — log error but don't fail (license already written)
    if let Err(e) = license_audit_repo::insert_audit(
        &state.db,
        &server_serial,
        &credential.user_name,
        &unit_name_clone,
        &credential.token_serial,
        &machine_fp_clone,
        &credential.cpu_id,
        &credential.board_serial,
        &payload_data.product,
        expires_at,
        &output_path,
        Some(&license_content),
    )
    .await
    {
        eprintln!("WARNING: audit insert failed (license was written): {}", e);
    }

    Ok(GenerateLicenseResult {
        success: true,
        output_path,
        machine_fp,
        error: None,
    })
}

/// List license audit entries for the history table.
#[tauri::command]
pub async fn list_license_audit(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>,
) -> Result<Vec<LicenseAuditEntry>, String> {
    let limit = limit.clamp(1, 200);
    let offset = offset.max(0);
    let rows = license_audit_repo::list_audit(&state.db, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| LicenseAuditEntry {
            id: r.id,
            server_serial: r.server_serial,
            user_name: r.user_name,
            unit_name: r.unit_name,
            token_serial: r.token_serial,
            machine_fp: r.machine_fp,
            product: r.product,
            expires_at: r.expires_at,
            created_at: r.created_at,
            license_blob: r.license_blob,
        })
        .collect())
}

/// Export a license file from stored DB blob to the standard output path.
#[tauri::command]
pub async fn export_license(
    audit_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let row = license_audit_repo::get_audit_by_id(&state.db, &audit_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Audit record not found")?;

    let blob = row.license_blob.ok_or("No license data stored for this record")?;

    // Resolve output path: SF\LICENSE\{safe_name}
    let safe_name = license_gen::sanitize_user_name(&row.user_name);
    let sub_path = format!("SF\\LICENSE\\{}", safe_name);
    let output_dir = crate::output_dir::resolve_output_dir(&state.db, None, &sub_path).await?;

    let file_name = format!("{}-license.dat", safe_name);
    let path = std::path::Path::new(&output_dir).join(&file_name);
    tokio::fs::write(&path, &blob)
        .await
        .map_err(|e| format!("Failed to write: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

/// Hard-delete a license audit record and its file from disk.
#[tauri::command]
pub async fn delete_license(
    audit_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let row = license_audit_repo::get_audit_by_id(&state.db, &audit_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Audit record not found")?;

    // Reconstruct expected path to prevent path traversal via tampered DB
    let safe_name = license_gen::sanitize_user_name(&row.user_name);
    let sub_path = format!("SF\\LICENSE\\{}", safe_name);
    let expected_dir = crate::output_dir::resolve_output_dir(&state.db, None, &sub_path).await?;
    let expected_file = format!("{}-license.dat", safe_name);
    let expected_path = std::path::Path::new(&expected_dir).join(&expected_file);

    // Delete file from disk (only the expected path, ignore if already gone)
    if expected_path.exists() {
        tokio::fs::remove_file(&expected_path)
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    // Delete DB record
    license_audit_repo::delete_audit(&state.db, &audit_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Open the LICENSE folder for a given user name in the system file explorer.
#[tauri::command]
pub async fn open_license_folder(
    user_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let safe_name = license_gen::sanitize_user_name(&user_name);
    let sub_path = format!("SF\\LICENSE\\{}", safe_name);
    let dir = crate::output_dir::resolve_output_dir(&state.db, None, &sub_path).await?;
    crate::commands::settings::open_folder(dir)
}

use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::{
    cert_parser::{self, CertInfo},
    db::{partner_members_repo, partners_repo},
    models::{PartnerMember, PartnerWithCount},
    AppState,
};

#[tauri::command]
pub async fn create_partner(
    name: String,
    app: AppHandle,  // C-3: needed to resolve data dir for SF subdirectory creation
    state: State<'_, AppState>,
) -> Result<PartnerWithCount, String> {
    let partner = partners_repo::create_partner(&state.db, &name)
        .await
        .map_err(|e| e.to_string())?;

    // C-3: auto-create SF subdirectories for this partner (errors silently ignored)
    if let Ok(data_dir) = app.path().app_data_dir() {
        let _ = std::fs::create_dir_all(data_dir.join("SF").join("ENCRYPT").join(&name));
        let _ = std::fs::create_dir_all(data_dir.join("SF").join("DECRYPT").join(&name));
    }

    Ok(PartnerWithCount {
        id: partner.id,
        name: partner.name,
        created_at: partner.created_at,
        member_count: 0,
    })
}

#[tauri::command]
pub async fn list_partners(state: State<'_, AppState>) -> Result<Vec<PartnerWithCount>, String> {
    partners_repo::list_partners(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_partner(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    partners_repo::rename_partner(&state.db, &id, &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_partner(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Collect cert file paths before DB delete (for filesystem cleanup)
    let members = partner_members_repo::list_members_by_partner(&state.db, &id)
        .await
        .map_err(|e| e.to_string())?;

    // CASCADE in DB schema deletes partner_member rows
    partners_repo::delete_partner(&state.db, &id)
        .await
        .map_err(|e| e.to_string())?;

    // Best-effort: delete cert files from AppData
    for m in members {
        let _ = std::fs::remove_file(&m.cert_file_path);
    }
    Ok(())
}

/// Parse a certificate file and return its metadata for user preview (no DB write)
#[tauri::command]
pub async fn import_cert_preview(cert_path: String) -> Result<CertInfo, String> {
    cert_parser::parse_cert_file(&cert_path)
}

/// Parse + copy cert file to AppData/DATA/Certs/partners/, then insert partner_member row in DB
#[tauri::command]
pub async fn add_partner_member(
    partner_id: String,
    cert_path: String,
    name: Option<String>,
    email: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<PartnerMember, String> {
    let cert_info = cert_parser::parse_cert_file(&cert_path)
        .map_err(|e| format!("Certificate parse error: {}", e))?;

    let member_name = name.unwrap_or_else(|| cert_info.cn.clone());
    let member_email = email.or(cert_info.email.clone());

    // Copy cert to AppData/DATA/Certs/partners/{uuid}.crt
    let certs_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("DATA")
        .join("Certs")
        .join("partners");
    std::fs::create_dir_all(&certs_dir)
        .map_err(|e| format!("Cannot create certs directory: {}", e))?;

    let cert_uuid = Uuid::new_v4().to_string();
    let dest = certs_dir.join(format!("{}.crt", cert_uuid));
    std::fs::copy(&cert_path, &dest)
        .map_err(|e| format!("Failed to copy certificate: {}", e))?;

    // Extract org from cert for storage
    let cert_org = cert_info.org.as_deref();

    // Insert DB row — rollback (delete copy) on failure
    let member = partner_members_repo::add_partner_member(
        &state.db,
        &partner_id,
        &member_name,
        member_email.as_deref(),
        &cert_info.cn,
        &cert_info.serial,
        cert_info.valid_from,
        cert_info.valid_to,
        &dest.to_string_lossy(),
        cert_org,
    )
    .await
    .map_err(|e| {
        let _ = std::fs::remove_file(&dest);
        e.to_string()
    })?;

    Ok(member)
}

#[tauri::command]
pub async fn list_partner_members(
    partner_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PartnerMember>, String> {
    partner_members_repo::list_members_by_partner(&state.db, &partner_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_partner_member(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let member = partner_members_repo::get_partner_member(&state.db, &id)
        .await
        .map_err(|e| e.to_string())?;

    partner_members_repo::delete_partner_member(&state.db, &id)
        .await
        .map_err(|e| e.to_string())?;

    // Best-effort cert file cleanup
    if let Some(m) = member {
        let _ = std::fs::remove_file(&m.cert_file_path);
    }
    Ok(())
}

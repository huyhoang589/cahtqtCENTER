pub mod app_log;
pub mod cert_parser;
pub mod commands;
pub mod db;
pub mod etoken;
pub mod ffi_helpers;
pub mod htqt_ffi;
pub mod lock_helper;
pub mod models;
pub mod output_dir;

use std::path::Path;
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;
use tauri::Manager;

use etoken::models::TokenLoginState;
use htqt_ffi::HtqtLib;

/// Shared application state managed by Tauri
pub struct AppState {
    pub db: SqlitePool,
    /// HTQT crypto DLL — None when htqt.dll not found at startup
    pub htqt_lib: Arc<Mutex<Option<HtqtLib>>>,
    /// Expected path of htqt.dll (computed once at startup from exe directory)
    pub dll_required_path: String,
    /// True while encrypt/decrypt batch is running — pauses PKCS#11 polling
    pub is_operation_running: Arc<Mutex<bool>>,
    /// Cached result of the last token_scan call (raw_der stored here, not sent to frontend)
    pub last_token_scan: Arc<Mutex<Option<etoken::models::TokenScanResult>>>,
    /// Verified token login state — holds PIN in Zeroizing<String> after successful login
    pub token_login: Arc<Mutex<TokenLoginState>>,
}

/// Create required DATA subdirectories under app_data_dir on startup (idempotent).
fn initialize_data_directories(app_data_dir: &Path) {
    let data = app_data_dir.join("DATA");
    let dirs = [
        data.clone(),
        data.join("Certs"),
        data.join("Certs").join("partners"),
        data.join("Certs").join("sender"),
        data.join("DB"),
        data.join("ENCRYPT"),
        data.join("DECRYPT"),
        data.join("LOGS"),
        data.join("CONFIG"),
    ];
    for dir in &dirs {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("Warning: cannot create {:?}: {}", dir, e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to resolve app data directory");

            // Initialize DATA subdirectories (best-effort, non-blocking)
            initialize_data_directories(&app_data_dir);

            // Initialize SQLite pool + run migrations
            let pool = tauri::async_runtime::block_on(async {
                db::init_db(&app_data_dir).await
                    .expect("Failed to initialize database")
            });

            // A-1: ensure output subdirectories exist under configured OUTPUT_DATA_DIR
            {
                use crate::db::settings_repo;
                let output_base = tauri::async_runtime::block_on(async {
                    settings_repo::get_all_settings(&pool).await.ok()
                })
                .and_then(|rows| {
                    rows.into_iter()
                        .find(|s| s.key == "output_data_dir")
                        .map(|s| s.value)
                })
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| {
                    std::env::var("USERPROFILE")
                        .map(|p| format!("{}/Desktop", p))
                        .unwrap_or_default()
                });
                let _ = commands::settings::create_output_subdirs(&output_base);
            }

            // Compute required DLL path once at startup (exe-adjacent htqt.dll)
            let dll_required_path = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.join("htqt.dll")))
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            // Try to load htqt.dll from exe directory (best-effort, exe-adjacent)
            let htqt_lib = if !dll_required_path.is_empty() {
                let p = std::path::Path::new(&dll_required_path);
                if p.exists() { HtqtLib::load(&dll_required_path).ok() } else { None }
            } else {
                None
            };

            app.manage(AppState {
                db: pool,
                htqt_lib: Arc::new(Mutex::new(htqt_lib)),
                dll_required_path,
                is_operation_running: Arc::new(Mutex::new(false)),
                last_token_scan: Arc::new(Mutex::new(None)),
                token_login: Arc::new(Mutex::new(TokenLoginState::default())),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::set_setting,
            commands::settings::get_app_info,
            commands::settings::get_app_settings,
            commands::settings_cert::import_sender_cert,
            commands::settings::open_folder,
            commands::settings::export_member_cert,
            commands::etoken::token_scan,
            commands::etoken::token_get_library_info,
            commands::etoken::token_export_sender_cert,
            commands::etoken::token_set_library_path,
            commands::etoken::token_clear_sender_cert,
            commands::etoken::login_token,
            commands::etoken::logout_token,
            commands::etoken::get_token_status,
            commands::partners::create_partner,
            commands::partners::list_partners,
            commands::partners::rename_partner,
            commands::partners::delete_partner,
            commands::partners::import_cert_preview,
            commands::partners::add_partner_member,
            commands::partners::list_partner_members,
            commands::partners::delete_partner_member,
            commands::encrypt::encrypt_batch,
            commands::decrypt::decrypt_batch,
            commands::communication::set_communication,
            commands::logs::list_logs,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running CAHTQT application");
}

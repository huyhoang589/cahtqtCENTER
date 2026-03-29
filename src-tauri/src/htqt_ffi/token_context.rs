use cryptoki::context::Pkcs11;
use cryptoki::object::{Attribute, ObjectClass, ObjectHandle};
use cryptoki::session::{Session, UserType};
use secrecy::Secret;
use tauri::AppHandle;

use crate::etoken::token_manager;

/// PKCS#11 session + private key handle + app context for DLL callbacks.
/// Owns both the Pkcs11 context and Session; Drop closes session then finalizes Pkcs11.
pub struct TokenContext {
    /// PKCS#11 library handle — Option so we can move it out in Drop (finalize takes self).
    pkcs11: Option<Pkcs11>,
    /// Active RW session — Option so we can explicitly drop before finalize.
    session: Option<Session>,
    /// Private key handle (CKA_SIGN=true) found on login.
    pub priv_key_handle: ObjectHandle,
    /// Tauri app handle for emitting progress events from callbacks.
    pub app: AppHandle,
    /// Sender's own certificate DER — used for SF v1 backward compatibility in decrypt.
    pub own_cert_der: Vec<u8>,
    /// Tauri event name to emit progress: "encrypt-progress" or "decrypt-progress".
    pub event_name: String,
}

impl Drop for TokenContext {
    fn drop(&mut self) {
        // Explicitly close session before C_Finalize — PKCS#11 spec requires this order.
        drop(self.session.take());
        // finalize() takes ownership; move out of Option before calling.
        if let Some(pkcs11) = self.pkcs11.take() {
            let _ = pkcs11.finalize();
        }
    }
}

impl TokenContext {
    /// Borrow the PKCS#11 session for callback use.
    /// Panics only if session was already dropped — impossible during a live DLL call.
    pub fn session(&self) -> &Session {
        self.session.as_ref().expect("TokenContext session must be open during DLL callbacks")
    }
}

/// Open a PKCS#11 RW session and login as User.
/// Creates its own Pkcs11 context (C_Initialize). The is_operation_running guard
/// prevents concurrent calls, so double-initialize is not an issue.
/// NOTE: caller must ensure AppState.token_login.pin holds the verified PIN.
pub fn open_token_session(
    pkcs11_lib_path: &str,
    slot_idx: u32,
    pin: &str,
    app: AppHandle,
    own_cert_der: Vec<u8>,
    event_name: String,
) -> Result<TokenContext, String> {
    // Initialize PKCS#11 library (C_Initialize — creates fresh context per command).
    let pkcs11 = token_manager::initialize(pkcs11_lib_path)?;

    let raw_slots = pkcs11
        .get_slots_with_token()
        .map_err(|e| format!("Slot enumeration failed: {}", e))?;

    let slot = raw_slots
        .get(slot_idx as usize)
        .ok_or_else(|| format!("Slot index {} out of range", slot_idx))?;

    let session = pkcs11
        .open_rw_session(*slot)
        .map_err(|e| format!("Failed to open RW session: {}", e))?;

    // C_Login — same pattern as login_token in etoken.rs.
    let auth_pin = Secret::new(pin.to_string());
    match session.login(UserType::User, Some(&auth_pin)) {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("CKR_USER_ALREADY_LOGGED_IN") {
                return Err(format!("PKCS#11 login failed: {}", msg));
            }
            // CKR_USER_ALREADY_LOGGED_IN treated as success
        }
    }

    // Find private key for signing (and decryption — same key handles both).
    let template = vec![
        Attribute::Class(ObjectClass::PRIVATE_KEY),
        Attribute::Sign(true),
    ];
    let keys = session
        .find_objects(&template)
        .map_err(|e| format!("Failed to find private key: {}", e))?;
    let priv_key = keys
        .first()
        .ok_or("No private signing key found on token")?
        .clone();

    Ok(TokenContext {
        pkcs11: Some(pkcs11),
        session: Some(session),
        priv_key_handle: priv_key,
        app,
        own_cert_der,
        event_name,
    })
}

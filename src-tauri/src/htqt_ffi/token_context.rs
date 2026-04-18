use std::sync::Arc;

use cryptoki::context::Pkcs11;
use cryptoki::object::{Attribute, ObjectClass, ObjectHandle};
use cryptoki::session::{Session, UserType};
use secrecy::Secret;
use tauri::AppHandle;

/// PKCS#11 session + private key handle + app context for DLL callbacks.
/// Holds Arc<Pkcs11> shared with AppState — C_Finalize deferred until logout_token drops the Arc.
pub struct TokenContext {
    /// Kept alive to defer C_Finalize until logout_token drops the last Arc (refcount → 0).
    _pkcs11: Arc<Pkcs11>,
    /// Active RW session — Option so we can explicitly drop before session ends.
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
        // Close session only — C_Finalize is deferred to logout_token via Arc lifecycle
        drop(self.session.take());
        // Arc<Pkcs11> drops automatically; C_Finalize triggers when last Arc is dropped (at logout)
    }
}

impl TokenContext {
    /// Borrow the PKCS#11 session for callback use.
    /// Panics only if session was already dropped — impossible during a live DLL call.
    pub fn session(&self) -> &Session {
        self.session.as_ref().expect("TokenContext session must be open during DLL callbacks")
    }
}

/// Open a PKCS#11 RW session using the persistent Pkcs11 context from AppState.
/// No C_Initialize — reuses the Arc<Pkcs11> kept alive from login_token to logout_token.
pub fn open_token_session(
    pkcs11: Arc<Pkcs11>,
    slot_idx: u32,
    pin: &str,
    app: AppHandle,
    own_cert_der: Vec<u8>,
    event_name: String,
) -> Result<TokenContext, String> {
    // pkcs11 is the persistent context from AppState — no C_Initialize here
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
        _pkcs11: pkcs11,
        session: Some(session),
        priv_key_handle: priv_key,
        app,
        own_cert_der,
        event_name,
    })
}

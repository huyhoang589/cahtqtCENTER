use std::ffi::c_void;
use std::panic::catch_unwind;
use std::slice;

use cryptoki::mechanism::rsa::{PkcsMgfType, PkcsOaepParams, PkcsOaepSource, PkcsPssParams};
use cryptoki::mechanism::{Mechanism, MechanismType};
use rand::thread_rng;
use rsa::pkcs8::DecodePublicKey;
use rsa::pss::{Signature as PssSignature, VerifyingKey};
use rsa::sha2::Sha256;
use rsa::signature::hazmat::PrehashVerifier;
use rsa::{Oaep, RsaPublicKey};
use serde::Serialize;
use tauri::Emitter;
use x509_parser::prelude::parse_x509_certificate;

use super::token_context::TokenContext;

// ---- Progress event payload emitted by cb_progress --------------------------

#[derive(Serialize, Clone)]
struct ProgressPayload {
    file_idx: u32,
    recip_idx: u32,
    file_total: u32,
    recip_total: u32,
    status: i32,
}

// ---- Callback implementations -----------------------------------------------

/// RSA-PSS-SHA256 sign: digest -> token hardware sign -> write signature.
/// Invoked by DLL during encrypt to sign each SF file.
/// Uses CKM_RSA_PKCS_PSS (pre-hashed) — DLL passes 32-byte SHA-256 digest.
pub unsafe extern "C" fn cb_rsa_pss_sign(
    digest: *const u8,
    digest_len: u32,
    signature: *mut u8,
    sig_len: *mut u32,
    user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if digest.is_null() || signature.is_null() || sig_len.is_null() || user_ctx.is_null() {
            return -1;
        }
        let ctx = &*(user_ctx as *const TokenContext);
        let digest_slice = slice::from_raw_parts(digest, digest_len as usize);

        // CKM_RSA_PKCS_PSS: pre-hashed sign with SHA-256 + MGF1-SHA256 + salt=32
        let pss_params = PkcsPssParams {
            hash_alg: MechanismType::SHA256,
            mgf: PkcsMgfType::MGF1_SHA256,
            s_len: 32_usize.try_into().expect("32 fits in Ulong"),
        };
        let mechanism = Mechanism::RsaPkcsPss(pss_params);

        match ctx.session().sign(&mechanism, ctx.priv_key_handle, digest_slice) {
            Ok(sig_bytes) => {
                let buf_capacity = *sig_len as usize;
                if sig_bytes.len() > buf_capacity {
                    eprintln!("[cb_sign] buffer too small: need {}, have {}", sig_bytes.len(), buf_capacity);
                    return -1;
                }
                std::ptr::copy_nonoverlapping(sig_bytes.as_ptr(), signature, sig_bytes.len());
                *sig_len = sig_bytes.len() as u32;
                0
            }
            Err(e) => {
                eprintln!("[cb_sign] PKCS#11 sign error: {}", e);
                -1
            }
        }
    });
    result.unwrap_or(-1)
}

/// RSA-OAEP-SHA256 encrypt plaintext with recipient's public key extracted from cert_der.
/// Invoked by DLL during encrypt for each recipient — software crypto (no token needed).
pub unsafe extern "C" fn cb_rsa_oaep_enc_cert(
    plaintext: *const u8,
    plaintext_len: u32,
    cert_der: *const u8,
    cert_der_len: u32,
    ciphertext_out: *mut u8,
    ciphertext_len: *mut u32,
    _user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if plaintext.is_null() || cert_der.is_null() || ciphertext_out.is_null() || ciphertext_len.is_null() {
            return -1;
        }
        let pt_slice = slice::from_raw_parts(plaintext, plaintext_len as usize);
        let cert_slice = slice::from_raw_parts(cert_der, cert_der_len as usize);

        let spki_der = match extract_spki_der(cert_slice) {
            Ok(d) => d,
            Err(e) => { eprintln!("[cb_enc_cert] cert parse: {}", e); return -1; }
        };
        let pub_key = match RsaPublicKey::from_public_key_der(&spki_der) {
            Ok(k) => k,
            Err(e) => { eprintln!("[cb_enc_cert] RSA key: {}", e); return -1; }
        };

        // Software RSA-OAEP-SHA256 encrypt (public key only — no token needed)
        let ciphertext = match pub_key.encrypt(&mut thread_rng(), Oaep::new::<Sha256>(), pt_slice) {
            Ok(ct) => ct,
            Err(e) => { eprintln!("[cb_enc_cert] OAEP encrypt: {}", e); return -1; }
        };

        let buf_capacity = *ciphertext_len as usize;
        if ciphertext.len() > buf_capacity {
            eprintln!("[cb_enc_cert] buffer too small: need {}, have {}", ciphertext.len(), buf_capacity);
            return -1;
        }
        std::ptr::copy_nonoverlapping(ciphertext.as_ptr(), ciphertext_out, ciphertext.len());
        *ciphertext_len = ciphertext.len() as u32;
        0
    });
    result.unwrap_or(-1)
}

/// RSA-OAEP-SHA256 decrypt ciphertext with token's private key.
/// Invoked by DLL during decrypt — token hardware operation.
pub unsafe extern "C" fn cb_rsa_oaep_decrypt(
    ciphertext: *const u8,
    ciphertext_len: u32,
    plaintext_out: *mut u8,
    plaintext_len: *mut u32,
    user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if ciphertext.is_null() || plaintext_out.is_null() || plaintext_len.is_null() || user_ctx.is_null() {
            return -1;
        }
        let ctx = &*(user_ctx as *const TokenContext);
        let ct_slice = slice::from_raw_parts(ciphertext, ciphertext_len as usize);

        // CKM_RSA_PKCS_OAEP with SHA-256 + MGF1-SHA256 + empty encoding param
        let oaep_params = PkcsOaepParams::new(
            MechanismType::SHA256,
            PkcsMgfType::MGF1_SHA256,
            PkcsOaepSource::empty(),
        );
        let mechanism = Mechanism::RsaPkcsOaep(oaep_params);

        match ctx.session().decrypt(&mechanism, ctx.priv_key_handle, ct_slice) {
            Ok(pt_bytes) => {
                let buf_capacity = *plaintext_len as usize;
                if pt_bytes.len() > buf_capacity {
                    eprintln!("[cb_decrypt] buffer too small: need {}, have {}", pt_bytes.len(), buf_capacity);
                    return -1;
                }
                std::ptr::copy_nonoverlapping(pt_bytes.as_ptr(), plaintext_out, pt_bytes.len());
                *plaintext_len = pt_bytes.len() as u32;
                0
            }
            Err(e) => { eprintln!("[cb_decrypt] PKCS#11 decrypt error: {}", e); -1 }
        }
    });
    result.unwrap_or(-1)
}

/// RSA-PSS-SHA256 verify sender's signature against pre-hashed digest using sender cert.
/// Invoked by DLL during decrypt — software crypto (public key from sender cert).
pub unsafe extern "C" fn cb_rsa_pss_verify(
    digest: *const u8,
    digest_len: u32,
    sig: *const u8,
    sig_len: u32,
    sender_cert_der: *const u8,
    sender_cert_der_len: u32,
    _user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if digest.is_null() || sig.is_null() || sender_cert_der.is_null() {
            return -1;
        }
        let digest_slice = slice::from_raw_parts(digest, digest_len as usize);
        let sig_slice = slice::from_raw_parts(sig, sig_len as usize);
        let cert_slice = slice::from_raw_parts(sender_cert_der, sender_cert_der_len as usize);

        let spki_der = match extract_spki_der(cert_slice) {
            Ok(d) => d,
            Err(e) => { eprintln!("[cb_verify] cert parse: {}", e); return -1; }
        };
        let pub_key = match RsaPublicKey::from_public_key_der(&spki_der) {
            Ok(k) => k,
            Err(e) => { eprintln!("[cb_verify] RSA key: {}", e); return -1; }
        };

        // Software RSA-PSS-SHA256 verify with pre-hashed digest
        let verifying_key = VerifyingKey::<Sha256>::new(pub_key);
        let pss_sig = match PssSignature::try_from(sig_slice) {
            Ok(s) => s,
            Err(e) => { eprintln!("[cb_verify] signature parse: {}", e); return -1; }
        };
        match verifying_key.verify_prehash(digest_slice, &pss_sig) {
            Ok(()) => 0,
            Err(e) => { eprintln!("[cb_verify] PSS verify failed: {}", e); -1 }
        }
    });
    result.unwrap_or(-1)
}

/// Progress callback: emit Tauri event per (file, recipient) pair.
/// event_name in TokenContext: "encrypt-progress" or "decrypt-progress".
pub unsafe extern "C" fn cb_progress(
    file_idx: u32,
    recip_idx: u32,
    file_total: u32,
    recip_total: u32,
    status: i32,
    user_ctx: *mut c_void,
) -> i32 {
    let result = catch_unwind(|| -> i32 {
        if user_ctx.is_null() { return 0; }
        let ctx = &*(user_ctx as *const TokenContext);
        let payload = ProgressPayload { file_idx, recip_idx, file_total, recip_total, status };
        let _ = ctx.app.emit(&ctx.event_name, payload);
        0 // never cancel
    });
    result.unwrap_or(0)
}

// ---- Helper -----------------------------------------------------------------

/// Parse cert DER and return the SubjectPublicKeyInfo DER bytes for use with RsaPublicKey.
fn extract_spki_der(cert_der: &[u8]) -> Result<Vec<u8>, String> {
    let (_, cert) = parse_x509_certificate(cert_der)
        .map_err(|e| format!("X.509 parse: {:?}", e))?;
    Ok(cert.public_key().raw.to_vec())
}

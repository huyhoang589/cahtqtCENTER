use cryptoki::object::{Attribute, AttributeType, ObjectClass};
use cryptoki::session::Session;
use sha1::{Digest, Sha1};
use x509_parser::prelude::*;

use crate::etoken::models::CertificateInfo;

/// Read all non-CA X.509 certificates from an open PKCS#11 session.
pub fn read_all_certificates(
    session: &Session,
    slot_id: u64,
) -> Result<Vec<CertificateInfo>, String> {
    let filter = vec![Attribute::Class(ObjectClass::CERTIFICATE)];
    let objects = session
        .find_objects(&filter)
        .map_err(|e| format!("Failed to find certificate objects (slot {}): {}", slot_id, e))?;

    let mut certs = Vec::new();
    for obj in objects {
        let attrs = match session.get_attributes(obj, &[
            AttributeType::Value,
            AttributeType::Label,
            AttributeType::Id,
        ]) {
            Ok(a) => a,
            Err(_) => continue,
        };

        let mut der_bytes: Option<Vec<u8>> = None;
        let mut label = String::new();
        let mut object_id = String::new();

        for attr in &attrs {
            match attr {
                Attribute::Value(der) => der_bytes = Some(der.clone()),
                Attribute::Label(lbl) => label = String::from_utf8_lossy(&lbl).to_string(),
                Attribute::Id(id) => {
                    object_id = id.iter().map(|b| format!("{:02X}", b)).collect();
                }
                _ => {}
            }
        }

        if let Some(der) = der_bytes {
            match parse_certificate(der, label, object_id) {
                Ok(cert_info) if !cert_info.is_ca => certs.push(cert_info),
                _ => {} // skip CA certs and parse errors
            }
        }
    }
    Ok(certs)
}

fn parse_certificate(
    der: Vec<u8>,
    label: String,
    object_id: String,
) -> Result<CertificateInfo, String> {
    let (_, cert) = parse_x509_certificate(&der)
        .map_err(|e| format!("X.509 parse error: {:?}", e))?;

    // SHA-1 fingerprint over raw DER bytes
    let fingerprint_sha1 = {
        let mut hasher = Sha1::new();
        hasher.update(&der);
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":")
    };

    // CA detection via BasicConstraints extension
    let is_ca = cert
        .basic_constraints()
        .ok()
        .flatten()
        .map(|bc| bc.value.ca)
        .unwrap_or(false);

    // Subject fields
    let subject_cn = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("")
        .to_string();

    let subject_org = cert
        .subject()
        .iter_organization()
        .next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("")
        .to_string();

    let subject_unit = cert
        .subject()
        .iter_organizational_unit()
        .next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("")
        .to_string();

    // Email from Subject Alternative Name (RFC 822) or skip if absent
    let subject_email = cert
        .subject_alternative_name()
        .ok()
        .flatten()
        .and_then(|ext| {
            ext.value.general_names.iter().find_map(|gn| {
                if let GeneralName::RFC822Name(addr) = gn {
                    Some(addr.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();

    // Issuer fields
    let issuer_cn = cert
        .issuer()
        .iter_common_name()
        .next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("")
        .to_string();

    let issuer_org = cert
        .issuer()
        .iter_organization()
        .next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("")
        .to_string();

    // Serial number as uppercase hex
    let serial_number: String = cert
        .raw_serial()
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect();

    // Validity dates as "YYYY-MM-DD" strings
    let not_before_ts = cert.validity().not_before.timestamp();
    let not_after_ts = cert.validity().not_after.timestamp();

    let valid_from = ts_to_date_str(not_before_ts);
    let valid_until = ts_to_date_str(not_after_ts);

    let now = chrono::Utc::now().timestamp();
    let is_expired = now > not_after_ts;

    Ok(CertificateInfo {
        object_id,
        label,
        subject_cn,
        subject_email,
        subject_org,
        subject_unit,
        issuer_cn,
        issuer_org,
        serial_number,
        valid_from,
        valid_until,
        is_expired,
        is_ca,
        key_usage: vec![], // optional for v1
        fingerprint_sha1,
        raw_der: der,
    })
}

fn ts_to_date_str(ts: i64) -> String {
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_opt(ts, 0)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

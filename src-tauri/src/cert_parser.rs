use serde::{Deserialize, Serialize};
use x509_parser::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub cn: String,
    pub email: Option<String>,
    pub org: Option<String>,
    pub serial: String,
    pub valid_from: i64,
    pub valid_to: i64,
    pub issuer_cn: Option<String>,
    pub file_path: Option<String>,
}

/// Parse a certificate from a file path (supports both PEM and DER formats)
pub fn parse_cert_file(path: &str) -> Result<CertInfo, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read certificate file: {}", e))?;
    parse_cert_bytes(&data)
}

/// Parse a certificate from raw bytes (auto-detects PEM vs DER)
pub fn parse_cert_bytes(data: &[u8]) -> Result<CertInfo, String> {
    // Detect PEM by looking for "-----BEGIN" header
    let der_owned: Vec<u8>;
    let der: &[u8] = if contains_pem_header(data) {
        let (_, pem) = x509_parser::pem::parse_x509_pem(data)
            .map_err(|e| format!("PEM parse error: {:?}", e))?;
        der_owned = pem.contents.clone();
        &der_owned
    } else {
        data
    };

    let (_, cert) = parse_x509_certificate(der)
        .map_err(|e| format!("Certificate parse error: {:?}", e))?;

    // Extract Common Name from Subject
    let cn = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    // Extract Organization from Subject
    let org = cert
        .subject()
        .iter_organization()
        .next()
        .and_then(|a| a.as_str().ok())
        .map(str::to_string);

    // Extract email from Subject Alternative Name extension
    let email = cert
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
        });

    // Extract Issuer Common Name
    let issuer_cn = cert
        .issuer()
        .iter_common_name()
        .next()
        .and_then(|a| a.as_str().ok())
        .map(str::to_string);

    // Serial number as uppercase hex string
    let serial: String = cert
        .raw_serial()
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect();

    let valid_from = cert.validity().not_before.timestamp();
    let valid_to = cert.validity().not_after.timestamp();

    Ok(CertInfo { cn, email, org, serial, valid_from, valid_to, issuer_cn, file_path: None })
}

fn contains_pem_header(data: &[u8]) -> bool {
    data.windows(b"-----BEGIN".len())
        .any(|w| w == b"-----BEGIN")
}

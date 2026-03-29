use std::path::Path;

use cryptoki::context::Pkcs11;

use crate::etoken::models::LibraryInfo;

pub struct LibraryCandidate {
    pub vendor: String,
    pub path: String,
}

/// Auto-detect PKCS#11 middleware library on Windows.
/// Priority: user-set path → bit4xpki → bit4opki → bit4ipki → eTPKCS11 → OpenSC
pub fn auto_detect_library(user_set_path: Option<&str>) -> Option<LibraryCandidate> {
    // User-specified path takes priority
    if let Some(p) = user_set_path {
        if !p.is_empty() && Path::new(p).exists() {
            return Some(LibraryCandidate {
                vendor: "Custom".to_string(),
                path: p.to_string(),
            });
        }
    }

    let candidates = [
        ("bit4ID",  r"C:\Windows\System32\bit4xpki.dll"),
        ("bit4ID",  r"C:\Windows\System32\bit4opki.dll"),
        ("bit4ID",  r"C:\Windows\System32\bit4ipki.dll"),
        ("SafeNet", r"C:\Windows\System32\eTPKCS11.dll"),
        ("OpenSC",  r"C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll"),
    ];

    for (vendor, path) in &candidates {
        if Path::new(path).exists() {
            return Some(LibraryCandidate {
                vendor: vendor.to_string(),
                path: path.to_string(),
            });
        }
    }
    None
}

/// Query the loaded PKCS#11 library for its version/description metadata.
pub fn get_library_info(pkcs11: &Pkcs11, vendor: &str, path: &str) -> Result<LibraryInfo, String> {
    let info = pkcs11
        .get_library_info()
        .map_err(|e| format!("Failed to get library info: {}", e))?;

    let cryptoki_ver = info.cryptoki_version();
    let library_ver = info.library_version();

    Ok(LibraryInfo {
        vendor: vendor.to_string(),
        description: info.library_description().trim().to_string(),
        path: path.to_string(),
        cryptoki_version: format!("{}.{}", cryptoki_ver.major(), cryptoki_ver.minor()),
        library_version: format!("{}.{}", library_ver.major(), library_ver.minor()),
        manufacturer_id: info.manufacturer_id().trim().to_string(),
    })
}

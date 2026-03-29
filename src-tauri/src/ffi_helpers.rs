use std::ffi::c_char;

/// Safely extract String from fixed-size c_char DLL output buffer.
/// Scans for null terminator; if DLL didn't null-terminate, stops at buffer end.
pub fn string_from_c_buf(buf: &[c_char]) -> String {
    let bytes: Vec<u8> = buf.iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

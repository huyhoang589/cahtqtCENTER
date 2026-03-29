pub mod callbacks;
pub mod error_codes;
pub mod lib_loader;
pub mod token_context;
pub mod types;

use std::sync::Mutex;

pub use error_codes::{htqt_error_message, htqt_error_name, HTQT_OK};
pub use lib_loader::HtqtLib;
pub use token_context::TokenContext;
pub use types::*;

/// Global lock — htqt_crypto.dll is NOT thread-safe (global state in DLL).
/// DLL_LOCK must be held across DLL calls + HTQT_GetError().
pub(crate) static DLL_LOCK: Mutex<()> = Mutex::new(());

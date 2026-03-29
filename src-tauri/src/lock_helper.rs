use std::sync::{Arc, Mutex, MutexGuard};

/// Safe mutex lock — recovers poisoned mutex instead of panicking.
pub fn safe_lock<T>(m: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    Ok(m.lock().unwrap_or_else(|poisoned| {
        eprintln!("WARN: recovered poisoned mutex");
        poisoned.into_inner()
    }))
}

/// RAII guard for is_operation_running flag.
/// Sets flag to true on creation, resets to false on drop.
pub struct OperationGuard {
    flag: Arc<Mutex<bool>>,
}

impl OperationGuard {
    pub fn acquire(flag: &Arc<Mutex<bool>>) -> Result<Self, String> {
        let mut g = safe_lock(flag)?;
        if *g {
            return Err("Operation already in progress".into());
        }
        *g = true;
        Ok(Self { flag: flag.clone() })
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        if let Ok(mut g) = self.flag.lock() {
            *g = false;
        }
    }
}

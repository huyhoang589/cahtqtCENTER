/// Emits an "app_log" Tauri event with level, message, and current timestamp.
use chrono::Local;
use tauri::{AppHandle, Emitter};
use crate::models::AppLogPayload;

pub fn emit_app_log(app: &AppHandle, level: &str, message: &str) {
    let _ = app.emit("app_log", AppLogPayload {
        level: level.to_string(),
        message: message.to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    });
}

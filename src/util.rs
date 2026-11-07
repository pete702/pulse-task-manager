//! Utility functions for PULSE.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Returns `v` if finite, otherwise `fallback`.
#[allow(dead_code)]
pub fn sanitize_f32(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Returns `v` if finite, otherwise `fallback`.
#[allow(dead_code)]
pub fn sanitize_f64(v: f64, fallback: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Interns process names so repeated names share one allocation.
pub fn intern_name(raw: &str) -> Arc<str> {
    static INTERN_POOL: OnceLock<Mutex<HashMap<String, Arc<str>>>> = OnceLock::new();
    let mut map = INTERN_POOL
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(v) = map.get(raw) {
        return Arc::clone(v);
    }
    if map.len() >= 512 {
        map.clear();
    }
    let v: Arc<str> = Arc::from(raw);
    map.insert(raw.to_string(), Arc::clone(&v));
    v
}

/// Formats uptime in seconds into a human-readable string (days, hours, minutes, seconds).
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 || days > 0 {
        parts.push(format!("{}h", hours));
    }
    if mins > 0 || hours > 0 || days > 0 {
        parts.push(format!("{}m", mins));
    }
    parts.push(format!("{}s", secs));

    parts.join(" ")
}

/// Formats current system time as HH:MM:SS using std::time (no chrono dependency).
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn format_timestamp_now() -> String {
    let mut st: windows_sys::Win32::Foundation::SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe {
        windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut st as *mut _);
    }
    format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond)
}

/// Formats current system time as HH:MM:SS using std::time (no chrono dependency).
#[cfg(not(target_os = "windows"))]
pub fn format_timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

//! Utility functions for PULSE.

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

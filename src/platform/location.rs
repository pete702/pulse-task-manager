//! Explorer file-location sanitization and verified open helpers.

use std::path::Path;

#[cfg(all(test, not(target_os = "windows")))]
use crate::platform::handle::verify_pid_safety_gate_ex;
#[cfg(all(test, target_os = "windows"))]
use crate::platform::handle::{
    is_handle_critical, query_process_creation_time, query_process_image_path,
    verify_pid_safety_gate_ex, SafeHandle,
};
#[cfg(all(test, target_os = "windows"))]
use crate::platform::protected::normalize_process_name;

/// Pure sanitizer/validator for Explorer file-location paths (no launch side effects).
pub fn sanitize_explorer_location_path(raw: &str) -> Result<String, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("Invalid executable path (empty)".to_string());
    }
    if s.chars().any(|c| c == '"' || c == '\0' || c.is_control()) {
        return Err("Invalid executable path (contains forbidden characters)".to_string());
    }
    if !Path::new(s).is_absolute() {
        return Err("Invalid executable path (relative)".to_string());
    }
    let normalized = normalize_explorer_path_root(s);
    if !Path::new(&normalized).exists() {
        return Err("Executable path does not exist".to_string());
    }
    Ok(normalized)
}

fn normalize_explorer_path_root(raw: &str) -> String {
    let mut s = raw.to_string();
    let is_drive_root = {
        let b = s.as_bytes();
        b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
    };
    if is_drive_root {
        while s.len() > 3 && s.ends_with('\\') {
            s.pop();
        }
        if s.len() == 2 {
            s.push('\\');
        }
    } else {
        while s.len() > 1 && (s.ends_with('\\') || s.ends_with('/')) {
            s.pop();
        }
    }
    s
}

/// Opens the containing folder of an executable in Explorer with the file selected.
pub fn open_process_file_location(exe_path: &Path) -> Result<(), String> {
    let raw = exe_path.to_string_lossy();
    let trimmed = sanitize_explorer_location_path(&raw)?;
    #[cfg(test)]
    {
        let _ = trimmed;
        Ok(())
    }
    #[cfg(not(test))]
    {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let system_root = std::env::var_os("SystemRoot")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"));
            std::process::Command::new(system_root.join("explorer.exe"))
                .raw_arg(format!("/select,\"{}\"", trimmed))
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("Failed to launch Explorer: {e}"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = trimmed;
            Err("open_process_file_location is only supported on Windows".to_string())
        }
    }
}

#[cfg(test)]
/// TOCTOU-safe open file location: verifies live (pid, creation_time) per candidate.
pub fn open_verified_process_location(
    candidates: &[(u32, Option<u64>)],
    expected_name: &str,
) -> Result<String, String> {
    if candidates.is_empty() {
        return Err("Open location failed: no verified target".to_string());
    }
    let mut any_actionable = false;
    for &(pid, creation_time) in candidates {
        let Some(ct) = creation_time.filter(|&t| t != 0) else {
            continue;
        };
        any_actionable = true;
        if let Some(path) = resolve_verified_live_path(pid, ct, expected_name) {
            open_process_file_location(Path::new(&path))?;
            return Ok(path);
        }
    }
    if !any_actionable {
        Err("Open location failed: missing or invalid process identity".to_string())
    } else {
        Err("Open location failed: no verified live target resolved".to_string())
    }
}

#[cfg(all(test, target_os = "windows"))]
fn resolve_verified_live_path(pid: u32, creation_time: u64, expected_name: &str) -> Option<String> {
    if pid == 0 || pid == 4 || pid == std::process::id() {
        return None;
    }
    let handle = SafeHandle::open_process(pid)?;
    if is_handle_critical(&handle) {
        return None;
    }
    if query_process_creation_time(&handle) != Some(creation_time) {
        return None;
    }
    let path = query_process_image_path(&handle)?;
    if path.trim().is_empty() {
        return None;
    }
    let live_norm = normalize_process_name(&path);
    if verify_pid_safety_gate_ex(pid, &live_norm, Some(&path)).is_err() {
        return None;
    }
    if live_norm != normalize_process_name(expected_name) {
        return None;
    }
    Some(path)
}

#[cfg(all(test, not(target_os = "windows")))]
fn resolve_verified_live_path(
    _pid: u32,
    _creation_time: u64,
    _expected_name: &str,
) -> Option<String> {
    None
}

//! Read-only Restart Manager file-lock discovery for dropped paths.
use std::path::PathBuf;

pub const MAX_LOCK_FILES: usize = 64;
#[cfg(target_os = "windows")]
const ERROR_MORE_DATA: u32 = 234;
const CWD_NOTE: &str = "Note: Restart Manager detects open file/DLL handles; folder cwd locks held by a shell are not reported by RM.";

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
#[link(name = "rstrtmgr")]
extern "system" {
    fn RmStartSession(session: *mut u32, flags: u32, key: *mut u16) -> u32;
    fn RmRegisterResources(
        session: u32,
        n_files: u32,
        files: *const *const u16,
        n_apps: u32,
        apps: *const RmUniqueProcess,
        n_services: u32,
        services: *const *const u16,
    ) -> u32;
    fn RmGetList(
        session: u32,
        needed: *mut u32,
        count: *mut u32,
        info: *mut RmProcessInfo,
        reboot_reasons: *mut u32,
    ) -> u32;
    fn RmEndSession(session: u32) -> u32;
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct RmUniqueProcess {
    pid: u32,
    start_time: windows_sys::Win32::Foundation::FILETIME,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct RmProcessInfo {
    process: RmUniqueProcess,
    app_name: [u16; 256],
    service_name: [u16; 64],
    app_type: i32,
    app_status: u32,
    session_id: u32,
    restartable: i32,
}

/// Result of a lock query: locking process identities and a human-readable summary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileLockReport {
    pub holders: Vec<(u32, String)>,
    pub summary: String,
}

impl FileLockReport {
    pub fn new(holders: Vec<(u32, String)>, had_directory: bool) -> Self {
        let summary = format_summary(&holders, had_directory);
        Self { holders, summary }
    }
}

/// Pure summary formatter; includes the cwd caveat for folders or empty results.
pub fn format_summary(holders: &[(u32, String)], had_directory: bool) -> String {
    if holders.is_empty() {
        return format!("No file lock holders detected. {CWD_NOTE}");
    }
    let mut text = format!("File locks held by {} process(es): ", holders.len());
    for (idx, (pid, name)) in holders.iter().enumerate() {
        if idx > 0 {
            text.push_str(", ");
        }
        text.push_str(&format!("{name} (PID {pid})"));
    }
    if had_directory {
        text.push(' ');
        text.push_str(CWD_NOTE);
    }
    text
}

/// Pure cap helper so capping can be tested without filesystem access.
pub fn cap_paths(paths: Vec<PathBuf>, cap: usize) -> Vec<PathBuf> {
    paths.into_iter().take(cap).collect()
}

/// Expands dropped files and directories into at most 64 concrete file paths.
pub fn collect_lock_files(dropped: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in dropped {
        if out.len() >= MAX_LOCK_FILES {
            break;
        }
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if out.len() >= MAX_LOCK_FILES {
                        break;
                    }
                    let child = entry.path();
                    if child.is_file() {
                        out.push(child);
                    }
                }
            }
        } else if path.is_file() {
            out.push(path.clone());
        }
    }
    cap_paths(out, MAX_LOCK_FILES)
}

/// Queries read-only lock holders for dropped paths; never mutates the processes.
#[allow(unsafe_code)]
pub fn find_file_locks(dropped_paths: &[PathBuf]) -> FileLockReport {
    let had_directory = dropped_paths.iter().any(|p| p.is_dir());
    let files = collect_lock_files(dropped_paths);
    #[cfg(target_os = "windows")]
    {
        if files.is_empty() {
            FileLockReport::new(Vec::new(), had_directory)
        } else {
            unsafe { windows_find_locks(&files, had_directory) }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = files;
        FileLockReport::new(Vec::new(), had_directory)
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn windows_find_locks(files: &[PathBuf], had_directory: bool) -> FileLockReport {
    use std::os::windows::ffi::OsStrExt;

    let mut session = 0u32;
    let mut key = [0u16; 32];
    if RmStartSession(&mut session, 0, key.as_mut_ptr()) != 0 {
        return FileLockReport::new(Vec::new(), had_directory);
    }
    struct SessionGuard(u32);
    impl Drop for SessionGuard {
        fn drop(&mut self) {
            unsafe {
                RmEndSession(self.0);
            }
        }
    }
    let _guard = SessionGuard(session);

    let wide: Vec<Vec<u16>> = files
        .iter()
        .map(|p| p.as_os_str().encode_wide().chain(std::iter::once(0)).collect())
        .collect();
    let ptrs: Vec<*const u16> = wide.iter().map(|v| v.as_ptr()).collect();
    if RmRegisterResources(
        session,
        ptrs.len() as u32,
        ptrs.as_ptr(),
        0,
        std::ptr::null(),
        0,
        std::ptr::null(),
    ) != 0
    {
        return FileLockReport::new(Vec::new(), had_directory);
    }

    let mut needed = 0u32;
    let mut count = 0u32;
    let mut reasons = 0u32;
    let rc = RmGetList(session, &mut needed, &mut count, std::ptr::null_mut(), &mut reasons);
    if (rc != 0 && rc != ERROR_MORE_DATA) || needed == 0 {
        return FileLockReport::new(Vec::new(), had_directory);
    }
    let capacity = needed.min(MAX_LOCK_FILES as u32) as usize;
    let mut info: Vec<RmProcessInfo> = (0..capacity).map(|_| std::mem::zeroed()).collect();
    count = capacity as u32;
    let rc = RmGetList(session, &mut needed, &mut count, info.as_mut_ptr(), &mut reasons);
    if rc != 0 && rc != ERROR_MORE_DATA {
        return FileLockReport::new(Vec::new(), had_directory);
    }
    let holders = info[..(count as usize).min(capacity)]
        .iter()
        .map(|entry| {
            let name = String::from_utf16_lossy(&entry.app_name).trim_end_matches('\0').to_owned();
            (entry.process.pid, name)
        })
        .collect();
    FileLockReport::new(holders, had_directory)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(n: usize) -> Vec<PathBuf> {
        (0..n).map(|i| PathBuf::from(format!("C:\\data\\file{i}.bin"))).collect()
    }

    #[test]
    fn cap_paths_limits_to_max_lock_files() {
        assert_eq!(cap_paths(paths(MAX_LOCK_FILES + 10), MAX_LOCK_FILES).len(), MAX_LOCK_FILES);
        assert_eq!(cap_paths(paths(3), MAX_LOCK_FILES).len(), 3);
        assert!(cap_paths(Vec::new(), MAX_LOCK_FILES).is_empty());
    }

    #[test]
    fn summary_lists_holders_with_pid() {
        let holders = vec![(4242, "app.exe".to_owned()), (99, "svc.exe".to_owned())];
        let report = FileLockReport::new(holders, false);
        assert_eq!(
            report.summary,
            "File locks held by 2 process(es): app.exe (PID 4242), svc.exe (PID 99)"
        );
        assert_eq!(report.holders.len(), 2);
    }

    #[test]
    fn summary_with_no_holders_includes_cwd_note() {
        let report = FileLockReport::new(Vec::new(), true);
        assert!(report.summary.starts_with("No file lock holders detected."));
        assert!(report
            .summary
            .contains("folder cwd locks held by a shell are not reported by RM."));
        assert!(report.holders.is_empty());
    }

    #[test]
    fn summary_with_holders_in_folder_includes_cwd_note() {
        let report = FileLockReport::new(vec![(7, "app.exe".to_owned())], true);
        assert!(report.summary.contains("app.exe (PID 7)"));
        assert!(report
            .summary
            .contains("folder cwd locks held by a shell are not reported by RM."));
    }
}

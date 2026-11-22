use crate::platform::sys_metrics::is_pulse_main_window_title;
use std::sync::atomic::{AtomicIsize, Ordering};

/// Handle of the acquired single-instance mutex (0 = none held).
static INSTANCE_MUTEX_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub struct SingleInstanceGuard;

pub enum AcquireOutcome {
    Primary(SingleInstanceGuard),
    AlreadyRunningWoken,
}

#[cfg(target_os = "windows")]
#[path = "instance_win.rs"]
mod instance_win;

pub fn acquire_single_instance() -> AcquireOutcome {
    #[cfg(target_os = "windows")]
    {
        instance_win::acquire()
    }
    #[cfg(not(target_os = "windows"))]
    {
        AcquireOutcome::Primary(SingleInstanceGuard)
    }
}

pub fn decide_already_exists_outcome(woken: bool) -> bool {
    woken
}

#[cfg(target_os = "windows")]
fn close_mutex_handle(handle: isize) {
    if handle != 0 {
        #[allow(unsafe_code)]
        unsafe {
            use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
            CloseHandle(handle as HANDLE);
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        close_mutex_handle(INSTANCE_MUTEX_HANDLE.swap(0, Ordering::SeqCst));
    }
}

/// Closes the held mutex so a restarted elevated instance can acquire it.
pub fn release_single_instance_mutex() {
    let handle = INSTANCE_MUTEX_HANDLE.swap(0, Ordering::SeqCst);
    #[cfg(target_os = "windows")]
    close_mutex_handle(handle);
    #[cfg(not(target_os = "windows"))]
    let _ = handle;
}

/// Pure check: a valid handle is required and ERROR_ALREADY_EXISTS means another owner.
pub(crate) fn reacquire_ok(handle: isize, last_error: u32) -> bool {
    const ERROR_ALREADY_EXISTS: u32 = 183;
    handle != 0 && last_error != ERROR_ALREADY_EXISTS
}

/// Re-acquires the mutex after a cancelled UAC prompt; true when held again.
pub fn reacquire_single_instance_mutex() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::GetLastError;
        use windows_sys::Win32::System::Threading::CreateMutexW;
        let name: Vec<u16> = "Local\\PulseTaskManagerSingleInstance\0".encode_utf16().collect();
        #[allow(unsafe_code)]
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) } as isize;
        #[allow(unsafe_code)]
        let err = unsafe { GetLastError() };
        if !reacquire_ok(handle, err) {
            close_mutex_handle(handle);
            return false;
        }
        INSTANCE_MUTEX_HANDLE.store(handle, Ordering::SeqCst);
        true
    }
    #[cfg(not(target_os = "windows"))]
    {
        INSTANCE_MUTEX_HANDLE.store(1, Ordering::SeqCst);
        true
    }
}

/// Restart arguments for the elevated relaunch (empty means no arguments).
pub fn build_restart_args(portable: bool) -> &'static str {
    if portable {
        "--portable"
    } else {
        ""
    }
}

/// `ShellExecuteW` reports success with a value strictly greater than 32.
pub fn shell_execute_succeeded(code: isize) -> bool {
    code > 32
}

/// Releases the instance mutex, relaunches elevated, and re-acquires on failure.
pub fn restart_as_admin(portable: bool) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        release_single_instance_mutex();
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => {
                if !reacquire_single_instance_mutex() {
                    crate::app_tray::mark_admin_restart_failed();
                }
                return false;
            }
        };
        let file: Vec<u16> =
            exe.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
        let verb: Vec<u16> = "runas\0".encode_utf16().collect();
        let params: Vec<u16> =
            build_restart_args(portable).encode_utf16().chain(std::iter::once(0)).collect();
        #[allow(unsafe_code)]
        let code = unsafe {
            ShellExecuteW(0, verb.as_ptr(), file.as_ptr(), params.as_ptr(), std::ptr::null(), 1)
        };
        if shell_execute_succeeded(code) {
            true
        } else {
            if !reacquire_single_instance_mutex() {
                crate::app_tray::mark_admin_restart_failed();
            }
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = build_restart_args(portable);
        let _ = shell_execute_succeeded(0);
        false
    }
}

pub fn basename_of(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

pub fn window_matches(
    current_pid: u32,
    candidate_pid: u32,
    title: &str,
    image_basename: &str,
) -> bool {
    candidate_pid != 0
        && candidate_pid != current_pid
        && is_pulse_main_window_title(title)
        && image_basename.eq_ignore_ascii_case("pulse_manager.exe")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::sys_metrics::is_pulse_main_window_title;

    #[test]
    fn instance_rules() {
        assert!(window_matches(200, 100, "PULSE", "pulse_manager.exe"));
        assert!(window_matches(200, 100, "\u{25B2} PULSE", "pulse_manager.exe"));
        assert!(!window_matches(200, 100, "", "pulse_manager.exe"));
        assert!(!window_matches(200, 100, "PulseTrayHost", "pulse_manager.exe"));
        assert!(!window_matches(200, 100, "PULSE", "other.exe"));
        assert!(!window_matches(200, 200, "PULSE", "pulse_manager.exe"));
        assert_eq!(basename_of("C:\\a\\pulse_manager.exe"), "pulse_manager.exe");
    }

    #[test]
    fn decide_already_exists_outcome_true() {
        assert!(decide_already_exists_outcome(true));
    }

    #[test]
    fn decide_already_exists_outcome_false() {
        assert!(!decide_already_exists_outcome(false));
    }

    #[test]
    fn shell_execute_success_threshold() {
        assert!(shell_execute_succeeded(33));
        assert!(!shell_execute_succeeded(32));
        assert!(!shell_execute_succeeded(5));
    }

    #[test]
    fn reacquire_rejects_already_exists_and_zero_handle() {
        assert!(reacquire_ok(42, 0));
        assert!(!reacquire_ok(42, 183));
        assert!(!reacquire_ok(0, 0));
        assert!(!reacquire_ok(0, 183));
    }

    #[test]
    fn restart_args_follow_portable_flag() {
        assert_eq!(build_restart_args(true), "--portable");
        assert_eq!(build_restart_args(false), "");
    }

    #[test]
    fn main_window_title_matches_both_real_titles() {
        assert!(is_pulse_main_window_title("\u{25B2} PULSE"));
        assert!(is_pulse_main_window_title("PULSE"));
        assert!(is_pulse_main_window_title("  \u{25B2} PULSE  "));
        assert!(!is_pulse_main_window_title(""));
        assert!(!is_pulse_main_window_title("PulseTrayHost"));
        assert!(!is_pulse_main_window_title("PULSE TRAY"));
    }
}

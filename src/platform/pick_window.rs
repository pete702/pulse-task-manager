//! Lightweight window-picker state machine for cross-process selection.
use std::sync::atomic::{AtomicBool, Ordering};

static ARMED: AtomicBool = AtomicBool::new(false);
const GA_ROOT: u32 = 2;
const VK_LEFT_BUTTON: i32 = 0x01;
const VK_ESCAPE: i32 = 0x1B;

#[cfg(target_os = "windows")]
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
#[link(name = "user32")]
extern "system" {
    fn GetAsyncKeyState(vkey: i32) -> i16;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn WindowFromPoint(point: Point) -> isize;
    fn GetAncestor(hwnd: isize, flags: u32) -> isize;
    fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
}

/// Arms the picker; a previous pending pick is discarded.
pub fn arm_pick_window() {
    ARMED.store(true, Ordering::SeqCst);
}

/// Cancels the picker without selecting anything.
pub fn cancel_pick_window() {
    ARMED.store(false, Ordering::SeqCst);
}

/// True while the picker is waiting for a mouse click.
pub fn is_pick_window_armed() -> bool {
    ARMED.load(Ordering::SeqCst)
}

/// Pure self-PID filter: zero and the caller's own PID are never selectable.
pub fn filter_picked_pid(pid: u32, self_pid: u32) -> Option<u32> {
    if pid != 0 && pid != self_pid {
        Some(pid)
    } else {
        None
    }
}

/// Polls for a click under the cursor and resolves the root window's PID.
pub fn poll_picked_window_pid(self_pid: u32) -> Option<u32> {
    if !is_pick_window_armed() {
        return None;
    }
    #[cfg(target_os = "windows")]
    {
        #[allow(unsafe_code)]
        unsafe {
            if GetAsyncKeyState(VK_ESCAPE) < 0 {
                cancel_pick_window();
                return None;
            }
            if GetAsyncKeyState(VK_LEFT_BUTTON) < 0 {
                let mut point = Point { x: 0, y: 0 };
                if GetCursorPos(&mut point) == 0 {
                    return None;
                }
                let hwnd = WindowFromPoint(point);
                let root = GetAncestor(hwnd, GA_ROOT);
                let mut pid = 0u32;
                GetWindowThreadProcessId(root, &mut pid);
                if let Some(external_pid) = filter_picked_pid(pid, self_pid) {
                    cancel_pick_window();
                    return Some(external_pid);
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = self_pid;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arm_and_cancel_transitions() {
        cancel_pick_window();
        assert!(!is_pick_window_armed());
        arm_pick_window();
        assert!(is_pick_window_armed());
        cancel_pick_window();
        assert!(!is_pick_window_armed());
    }

    #[test]
    fn filter_picked_pid_rejects_zero_and_self() {
        assert_eq!(filter_picked_pid(1234, 999), Some(1234));
        assert_eq!(filter_picked_pid(0, 999), None);
        assert_eq!(filter_picked_pid(999, 999), None);
    }
}

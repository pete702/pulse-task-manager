//! Win32 self-maintenance helpers for the PULSE process itself.

/// Trims PULSE's own working set (font atlas / OpenGL buffers).
/// Returns `true` when the OS accepted the trim request.
pub fn self_trim_working_set() -> bool {
    crate::modules::memory::win32_api::trim_own_working_set()
}

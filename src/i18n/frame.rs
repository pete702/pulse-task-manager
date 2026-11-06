//! Frame/window chrome translations and formatting for PULSE i18n.
//! **100% English Code** — Ukrainian translations removed.

use super::{Key, Lang};

impl Lang {
    /// Translates frame/window chrome keys for this language.
    pub(super) fn translate_frame(self, key: Key) -> Option<&'static str> {
        match (self, key) {
            // Top bar / window chrome
            (Lang::En, Key::AppTitle) => Some("▲ PULSE // TELEMETRY"),
            (Lang::En, Key::VersionPrefix) => Some("v"),
            (Lang::En, Key::UptimeLabel) => Some("Uptime"),

            // Sidebar tabs (geometric Unicode symbols, no emoji)
            (Lang::En, Key::TabMemory) => Some("[01] Memory (RAM)"),
            (Lang::En, Key::TabCpu) => Some("[02] Processor (CPU)"),
            (Lang::En, Key::TabStorage) => Some("[03] Storage (Disks)"),
            (Lang::En, Key::TabNetwork) => Some("[04] Network & Ping"),
            (Lang::En, Key::TabSystem) => Some("[05] System & GPU"),

            // Central panel
            (Lang::En, Key::AwaitingMemoryModel) => {
                Some("Telemetry core active. Awaiting memory model...")
            }

            // Self-test
            (Lang::En, Key::SelfTestPassedHeadless) => Some("SELF-TEST PASSED (HEADLESS CI)"),
            (Lang::En, Key::SelfTestPassedLive) => Some("SELF-TEST PASSED (LIVE GUI)"),
            (Lang::En, Key::SelfTestFailed) => Some("SELF-TEST FAILED"),

            // Status bar
            (Lang::En, Key::StatusReady) => Some("● READY"),

            _ => None,
        }
    }
}

/// Locale badge strings for the top bar (OS locale detection).
pub mod locale {
    #[cfg(target_os = "windows")]
    use std::sync::OnceLock;

    #[cfg(target_os = "windows")]
    #[allow(unsafe_code)]
    fn real_locale_upper() -> String {
        static CACHE: OnceLock<String> = OnceLock::new();
        CACHE
            .get_or_init(|| {
                let mut buf = [0u16; 32];
                use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;
                let len = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), 32) };
                if len > 1 && (len as usize) <= buf.len() {
                    let raw = String::from_utf16_lossy(&buf[..(len as usize - 1)]);
                    let up = raw.to_ascii_uppercase();
                    if !up.is_empty() {
                        return up;
                    }
                }
                "EN-US".to_owned()
            })
            .clone()
    }

    /// Short locale badge for top bar (e.g., "LOCALE: UK-UA" without hex LCID).
    /// Uses OnceLock with static string references - zero allocations after init.
    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    pub fn os_locale_badge() -> &'static str {
        static BADGE_CACHE: OnceLock<&'static str> = OnceLock::new();
        BADGE_CACHE
            .get_or_init(|| Box::leak(format!("LOCALE: {}", real_locale_upper()).into_boxed_str()))
    }

    /// Full locale string for hover tooltip (e.g., "System Locale: UK-UA").
    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    pub fn os_locale_full() -> &'static str {
        static FULL_CACHE: OnceLock<&'static str> = OnceLock::new();
        FULL_CACHE.get_or_init(|| {
            Box::leak(format!("System Locale: {}", real_locale_upper()).into_boxed_str())
        })
    }

    /// Bare short code for top bar (e.g., "UK-UA").
    #[cfg(target_os = "windows")]
    pub fn os_locale_short() -> &'static str {
        static SHORT_CACHE: OnceLock<&'static str> = OnceLock::new();
        SHORT_CACHE.get_or_init(|| Box::leak(real_locale_upper().into_boxed_str()))
    }

    /// Short badge fallback for non-Windows targets.
    #[cfg(not(target_os = "windows"))]
    pub fn os_locale_badge() -> &'static str {
        "LOCALE: EN-US"
    }

    /// Full fallback for non-Windows targets.
    #[cfg(not(target_os = "windows"))]
    pub fn os_locale_full() -> &'static str {
        "System Locale: EN-US"
    }

    /// Bare short fallback for non-Windows targets.
    #[cfg(not(target_os = "windows"))]
    pub fn os_locale_short() -> &'static str {
        "EN-US"
    }
}

/// Formats the system-cache subtitle, e.g. "OS Reserve • 13.4% of RAM".
#[allow(dead_code)]
pub fn system_cache_subtitle(pct: f64) -> String {
    format!("OS Reserve • {pct:.1}% of RAM")
}

/// Formats the kernel pool paged/non-paged detail line.
#[allow(dead_code)]
pub fn kernel_pool_detail(paged: &str, nonpaged: &str) -> String {
    format!("Paged: {paged} / Non-paged: {nonpaged}")
}

/// Formats the commit limit line with its usage percentage.
#[allow(dead_code)]
pub fn commit_limit_label(limit: &str, pct: f32) -> String {
    format!("Limit: {limit} ({pct:.1}%)")
}

/// Formats the OS activity counts (processes, threads, handles).
#[allow(dead_code)]
pub fn os_activity(processes: u32, threads: u32, handles: u32) -> String {
    format!("{processes} processes | {threads} threads | {handles} handles")
}

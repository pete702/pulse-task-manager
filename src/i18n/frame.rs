//! Frame/window chrome translations and formatting for PULSE i18n.
//! **100% English Code** — Ukrainian translations removed.

use super::{Key, Lang};

impl Lang {
    /// Translates frame/window chrome keys for this language.
    pub(super) fn translate_frame(self, key: Key) -> Option<&'static str> {
        match (self, key) {
            // Top bar / window chrome
            (Lang::En, Key::AppTitle) => Some("⚡ PULSE // TELEMETRY"),
            (Lang::En, Key::VersionPrefix) => Some("v"),
            (Lang::En, Key::UptimeLabel) => Some("Uptime"),

            // Sidebar tabs (geometric Unicode symbols, no emoji)
            (Lang::En, Key::TabMemory) => Some("◈ Memory (RAM)"),
            (Lang::En, Key::TabCpu) => Some("▣ Processor (CPU)"),
            (Lang::En, Key::TabStorage) => Some("◧ Storage (Disks)"),
            (Lang::En, Key::TabNetwork) => Some("⇅ Network & Ping"),
            (Lang::En, Key::TabSystem) => Some("⚙ System & GPU"),

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
    use std::sync::OnceLock;

    #[cfg(target_os = "windows")]
    static OS_LANG_ID: OnceLock<u16> = OnceLock::new();

    #[cfg(target_os = "windows")]
    #[allow(unsafe_code)]
    fn get_os_lang_id() -> u16 {
        *OS_LANG_ID.get_or_init(|| {
            use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
            unsafe { GetUserDefaultUILanguage() }
        })
    }

    /// Returns the OS locale badge string for the top bar (e.g., "LOCALE: UK-UA (0x0422)").
    /// Uses OnceLock to cache the Win32 GetUserDefaultUILanguage() result - zero FFI calls after first call.
    #[cfg(target_os = "windows")]
    pub fn os_locale_badge() -> &'static str {
        static BADGE_CACHE: OnceLock<String> = OnceLock::new();
        BADGE_CACHE.get_or_init(|| {
            let lang_id = get_os_lang_id();
            let primary_lang = lang_id & 0x3FF;
            if primary_lang == 0x22 {
                "LOCALE: UK-UA (0x0422)".to_string()
            } else {
                "LOCALE: EN-US (0x0409)".to_string()
            }
        })
    }

    /// Fallback for non-Windows targets (should not happen for this Windows-only app).
    #[cfg(not(target_os = "windows"))]
    pub fn os_locale_badge() -> &'static str {
        "LOCALE: EN-US (0x0409)"
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

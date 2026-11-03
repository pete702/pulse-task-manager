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
            (Lang::En, Key::SelfDiagLabel) => Some("Self-Diag"),
            (Lang::En, Key::LocaleIndicator) => Some("Locale"),

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
            (Lang::En, Key::StatusInfo) => {
                Some("PULSE v0.0.11  |  Windows Kernel Monitor  |  Active Processes: {}")
            }

            _ => None,
        }
    }
}

/// Locale badge strings for the top bar (OS locale detection).
pub mod locale {
    /// Returns the OS locale badge string for the top bar (e.g., "LOCALE: UK-UA (0x0422)").
    #[cfg(target_os = "windows")]
    #[allow(unsafe_code)]
    pub fn os_locale_badge() -> &'static str {
        use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
        let lang_id = unsafe { GetUserDefaultUILanguage() };
        let primary_lang = lang_id & 0x3FF;
        if primary_lang == 0x22 {
            "LOCALE: UK-UA (0x0422)"
        } else {
            "LOCALE: EN-US (0x0409)"
        }
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

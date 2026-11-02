//! Internationalization (i18n) for PULSE: auto-detect Windows UI language (UA/EN).

use std::sync::atomic::{AtomicU8, Ordering};

/// Supported interface languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    /// Ukrainian (default for owner).
    #[default]
    Uk,
    /// English (for GitHub/international).
    En,
}

impl Lang {
    /// Current language indicator for display.
    #[allow(dead_code)]
    #[inline]
    pub fn current_label(self) -> &'static str {
        match self {
            Lang::Uk => "UA",
            Lang::En => "EN",
        }
    }

    /// Returns the locale string for display in top bar.
    #[inline]
    pub fn locale_string(self) -> &'static str {
        match self {
            Lang::Uk => "UK-UA (0x0422)",
            Lang::En => "EN-US (0x0409)",
        }
    }
}

/// Thread-safe global language state.
/// Uses AtomicU8 for lock-free reads/writes (0 = Uk, 1 = En).
static CURRENT_LANG: AtomicU8 = AtomicU8::new(Lang::Uk as u8);

/// Gets the current global language.
#[inline]
pub fn current_lang() -> Lang {
    match CURRENT_LANG.load(Ordering::Relaxed) {
        0 => Lang::Uk,
        _ => Lang::En,
    }
}

/// Sets the global language.
#[inline]
pub fn set_lang(lang: Lang) {
    CURRENT_LANG.store(lang as u8, Ordering::Relaxed);
}

/// Initializes language from OS detection (Windows GetUserDefaultUILanguage).
/// Called once at startup. Can be overridden by `--lang=uk|en` CLI arg for testing.
pub fn init_language(cli_lang_override: Option<&str>) {
    let lang = if let Some(override_str) = cli_lang_override {
        match override_str {
            "uk" => Lang::Uk,
            "en" => Lang::En,
            _ => detect_os_language().0,
        }
    } else {
        detect_os_language().0
    };
    set_lang(lang);
}

/// Detects the Windows UI language via GetUserDefaultUILanguage().
/// Returns (Lang, lang_id) where lang_id is the raw LANGID (e.g., 0x0422 for Ukrainian).
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn detect_os_language() -> (Lang, u16) {
    use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

    // SAFETY: GetUserDefaultUILanguage is a simple Win32 API call with no unsafe preconditions.
    let lang_id = unsafe { GetUserDefaultUILanguage() };

    // Extract primary language ID (lower 10 bits)
    // LANG_UKRAINIAN = 0x22, LANG_ENGLISH = 0x09
    let primary_lang = lang_id & 0x3FF;

    let lang = if primary_lang == 0x22 {
        // LANG_UKRAINIAN
        Lang::Uk
    } else {
        Lang::En
    };

    (lang, lang_id)
}

/// Fallback for non-Windows targets (should not happen for this Windows-only app).
#[cfg(not(target_os = "windows"))]
pub fn detect_os_language() -> (Lang, u16) {
    (Lang::En, 0x0409)
}

/// Translation key type for type-safe lookups.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // Top bar
    AppTitle,
    VersionPrefix,
    UptimeLabel,
    SelfDiagLabel,
    LocaleIndicator,

    // Sidebar tabs
    TabMemory,
    TabCpu,
    TabStorage,
    TabNetwork,
    TabSystem,

    // Central panel
    AwaitingMemoryModel,

    // Self-test
    SelfTestPassedHeadless,
    SelfTestPassedLive,
    SelfTestFailed,

    // Memory module - Card headers
    PhysicalRamTitle,
    SwapCommitTitle,

    // Memory module - Grid labels
    UsedTotal,
    AvailableFree,
    CachedStandby,
    PeakSession,
    SwapUsedTotal,
    CommitChargeLimit,
    CommitPercent,

    // Kernel mini-tiles
    SystemCache,
    KernelPool,
    CommitCharge,
    OsActivity,

    // Chart
    MemoryHistory60s,
    RamPercent,
    SwapPercent,

    // Status bar
    StatusReady,
    StatusInfo,

    // Memory module - legacy keys (keep for backward compat)
    MemoryTotal,
    MemoryAvailable,
    MemoryUsed,
    SwapTotal,
    SwapUsed,
    SwapFree,
    ClearCacheButton,
    CacheCleared,
    CacheClearFailed,
}

impl Key {
    /// Returns the translated string for the current global language.
    #[inline]
    pub fn t(self) -> &'static str {
        current_lang().translate(self)
    }

    /// Returns the translated string for a specific language.
    #[allow(dead_code)]
    #[inline]
    pub fn t_lang(self, lang: Lang) -> &'static str {
        lang.translate(self)
    }
}

impl Lang {
    /// Translates a key for this language.
    fn translate(self, key: Key) -> &'static str {
        match (self, key) {
            // Top bar
            (Lang::Uk, Key::AppTitle) => "⚡ PULSE // TELEMETRY",
            (Lang::En, Key::AppTitle) => "⚡ PULSE // TELEMETRY",

            (Lang::Uk, Key::VersionPrefix) => "v",
            (Lang::En, Key::VersionPrefix) => "v",

            (Lang::Uk, Key::UptimeLabel) => "Uptime",
            (Lang::En, Key::UptimeLabel) => "Uptime",

            (Lang::Uk, Key::SelfDiagLabel) => "Self-Diag",
            (Lang::En, Key::SelfDiagLabel) => "Self-Diag",

            (Lang::Uk, Key::LocaleIndicator) => "Локаль",
            (Lang::En, Key::LocaleIndicator) => "Locale",

            // Sidebar tabs (geometric Unicode symbols, no emoji)
            (Lang::Uk, Key::TabMemory) => "◈ Пам'ять (RAM)",
            (Lang::En, Key::TabMemory) => "◈ Memory (RAM)",

            (Lang::Uk, Key::TabCpu) => "▣ Процесор (CPU)",
            (Lang::En, Key::TabCpu) => "▣ Processor (CPU)",

            (Lang::Uk, Key::TabStorage) => "◧ Диски (Storage)",
            (Lang::En, Key::TabStorage) => "◧ Storage (Disks)",

            (Lang::Uk, Key::TabNetwork) => "⇅ Мережа (Network)",
            (Lang::En, Key::TabNetwork) => "⇅ Network & Ping",

            (Lang::Uk, Key::TabSystem) => "⚙ Система та GPU",
            (Lang::En, Key::TabSystem) => "⚙ System & GPU",

            // Central panel
            (Lang::Uk, Key::AwaitingMemoryModel) => {
                "Ядро телеметрії активне. Очікування моделі пам'яті..."
            }
            (Lang::En, Key::AwaitingMemoryModel) => {
                "Telemetry core active. Awaiting memory model..."
            }

            // Self-test
            (Lang::Uk, Key::SelfTestPassedHeadless) => "SELF-TEST PASSED (HEADLESS CI)",
            (Lang::En, Key::SelfTestPassedHeadless) => "SELF-TEST PASSED (HEADLESS CI)",

            (Lang::Uk, Key::SelfTestPassedLive) => "SELF-TEST PASSED (LIVE GUI)",
            (Lang::En, Key::SelfTestPassedLive) => "SELF-TEST PASSED (LIVE GUI)",

            (Lang::Uk, Key::SelfTestFailed) => "SELF-TEST FAILED",
            (Lang::En, Key::SelfTestFailed) => "SELF-TEST FAILED",

            // Memory module - Card headers
            (Lang::Uk, Key::PhysicalRamTitle) => "◈ ФІЗИЧНА ПАМ'ЯТЬ (RAM)",
            (Lang::En, Key::PhysicalRamTitle) => "◈ PHYSICAL MEMORY (RAM)",

            (Lang::Uk, Key::SwapCommitTitle) => "▣ ФАЙЛ ПІДКАЧКИ ТА COMMIT",
            (Lang::En, Key::SwapCommitTitle) => "▣ SWAP FILE & COMMIT",

            // Memory module - Grid labels
            (Lang::Uk, Key::UsedTotal) => "Використано / Всього",
            (Lang::En, Key::UsedTotal) => "Used / Total",

            (Lang::Uk, Key::AvailableFree) => "Доступно / Вільно",
            (Lang::En, Key::AvailableFree) => "Available / Free",

            (Lang::Uk, Key::CachedStandby) => "Кешовано (Standby)",
            (Lang::En, Key::CachedStandby) => "Cached (Standby)",

            (Lang::Uk, Key::PeakSession) => "Пік за сесію",
            (Lang::En, Key::PeakSession) => "Peak (Session)",

            (Lang::Uk, Key::SwapUsedTotal) => "Swap Використано / Всього",
            (Lang::En, Key::SwapUsedTotal) => "Swap Used / Total",

            (Lang::Uk, Key::CommitChargeLimit) => "Commit Charge / Limit",
            (Lang::En, Key::CommitChargeLimit) => "Commit Charge / Limit",

            (Lang::Uk, Key::CommitPercent) => "Commit %",
            (Lang::En, Key::CommitPercent) => "Commit %",

            // Kernel mini-tiles
            (Lang::Uk, Key::SystemCache) => "Системний кеш",
            (Lang::En, Key::SystemCache) => "System Cache",

            (Lang::Uk, Key::KernelPool) => "Пул ядра",
            (Lang::En, Key::KernelPool) => "Kernel Pool",

            (Lang::Uk, Key::CommitCharge) => "Виділено (Commit Charge)",
            (Lang::En, Key::CommitCharge) => "Committed (Commit Charge)",

            (Lang::Uk, Key::OsActivity) => "Активність ОС (Handles & Threads)",
            (Lang::En, Key::OsActivity) => "OS Activity (Handles & Threads)",

            // Chart
            (Lang::Uk, Key::MemoryHistory60s) => "● 60-SECOND MEMORY HISTORY",
            (Lang::En, Key::MemoryHistory60s) => "● 60-SECOND MEMORY HISTORY",

            (Lang::Uk, Key::RamPercent) => "RAM %",
            (Lang::En, Key::RamPercent) => "RAM %",

            (Lang::Uk, Key::SwapPercent) => "Swap %",
            (Lang::En, Key::SwapPercent) => "Swap %",

            // Status bar
            (Lang::Uk, Key::StatusReady) => "● ГОТОВО",
            (Lang::En, Key::StatusReady) => "● READY",

            (Lang::Uk, Key::StatusInfo) => {
                "PULSE v0.0.7  |  Obsidian HUD  |  60s History  |  Windows Kernel Metrics"
            }
            (Lang::En, Key::StatusInfo) => {
                "PULSE v0.0.7  |  Obsidian HUD  |  60s History  |  Windows Kernel Metrics"
            }

            // Memory module - legacy keys (keep for backward compat)
            (Lang::Uk, Key::MemoryTotal) => "Всього RAM",
            (Lang::En, Key::MemoryTotal) => "Total RAM",

            (Lang::Uk, Key::MemoryAvailable) => "Доступно",
            (Lang::En, Key::MemoryAvailable) => "Available",

            (Lang::Uk, Key::MemoryUsed) => "Використано",
            (Lang::En, Key::MemoryUsed) => "Used",

            (Lang::Uk, Key::SwapTotal) => "Всього Swap",
            (Lang::En, Key::SwapTotal) => "Total Swap",

            (Lang::Uk, Key::SwapUsed) => "Використано Swap",
            (Lang::En, Key::SwapUsed) => "Swap Used",

            (Lang::Uk, Key::SwapFree) => "Вільно Swap",
            (Lang::En, Key::SwapFree) => "Swap Free",

            (Lang::Uk, Key::ClearCacheButton) => "Очистити кеш",
            (Lang::En, Key::ClearCacheButton) => "Clear Cache",

            (Lang::Uk, Key::CacheCleared) => "Кеш очищено",
            (Lang::En, Key::CacheCleared) => "Cache cleared",

            (Lang::Uk, Key::CacheClearFailed) => "Помилка очищення кешу",
            (Lang::En, Key::CacheClearFailed) => "Cache clear failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lang_default_is_uk() {
        assert_eq!(Lang::default(), Lang::Uk);
    }

    #[test]
    fn test_current_lang_get_set() {
        set_lang(Lang::En);
        assert_eq!(current_lang(), Lang::En);
        set_lang(Lang::Uk);
        assert_eq!(current_lang(), Lang::Uk);
    }

    #[test]
    fn test_init_language_override() {
        init_language(Some("uk"));
        assert_eq!(current_lang(), Lang::Uk);
        init_language(Some("en"));
        assert_eq!(current_lang(), Lang::En);
    }

    #[test]
    fn test_key_translation_uk() {
        set_lang(Lang::Uk);
        assert_eq!(Key::AppTitle.t(), "⚡ PULSE // TELEMETRY");
        assert_eq!(Key::TabMemory.t(), "◈ Пам'ять (RAM)");
        assert_eq!(Key::TabCpu.t(), "▣ Процесор (CPU)");
        assert_eq!(Key::TabStorage.t(), "◧ Диски (Storage)");
        assert_eq!(Key::TabNetwork.t(), "⇅ Мережа (Network)");
        assert_eq!(Key::TabSystem.t(), "⚙ Система та GPU");
        assert_eq!(Key::UptimeLabel.t(), "Uptime");
        assert_eq!(Key::PhysicalRamTitle.t(), "◈ ФІЗИЧНА ПАМ'ЯТЬ (RAM)");
        assert_eq!(Key::SwapCommitTitle.t(), "▣ ФАЙЛ ПІДКАЧКИ ТА COMMIT");
        assert_eq!(Key::UsedTotal.t(), "Використано / Всього");
        assert_eq!(Key::AvailableFree.t(), "Доступно / Вільно");
        assert_eq!(Key::CachedStandby.t(), "Кешовано (Standby)");
        assert_eq!(Key::PeakSession.t(), "Пік за сесію");
    }

    #[test]
    fn test_key_translation_en() {
        set_lang(Lang::En);
        assert_eq!(Key::AppTitle.t(), "⚡ PULSE // TELEMETRY");
        assert_eq!(Key::TabMemory.t(), "◈ Memory (RAM)");
        assert_eq!(Key::TabCpu.t(), "▣ Processor (CPU)");
        assert_eq!(Key::TabStorage.t(), "◧ Storage (Disks)");
        assert_eq!(Key::TabNetwork.t(), "⇅ Network & Ping");
        assert_eq!(Key::TabSystem.t(), "⚙ System & GPU");
        assert_eq!(Key::UptimeLabel.t(), "Uptime");
        assert_eq!(Key::PhysicalRamTitle.t(), "◈ PHYSICAL MEMORY (RAM)");
        assert_eq!(Key::SwapCommitTitle.t(), "▣ SWAP FILE & COMMIT");
        assert_eq!(Key::UsedTotal.t(), "Used / Total");
        assert_eq!(Key::AvailableFree.t(), "Available / Free");
        assert_eq!(Key::CachedStandby.t(), "Cached (Standby)");
        assert_eq!(Key::PeakSession.t(), "Peak (Session)");
    }

    #[test]
    fn test_locale_strings() {
        assert_eq!(Lang::Uk.locale_string(), "UK-UA (0x0422)");
        assert_eq!(Lang::En.locale_string(), "EN-US (0x0409)");
    }

    #[test]
    fn test_detect_os_language() {
        // This test just verifies the function exists and returns a valid tuple
        let (lang, lang_id) = detect_os_language();
        assert!(matches!(lang, Lang::Uk | Lang::En));
        assert_ne!(lang_id, 0);
    }
}

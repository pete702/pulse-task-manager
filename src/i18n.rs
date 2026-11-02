//! Internationalization (i18n) for PULSE: language switching UA/EN.

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
    /// Returns the other language (toggle).
    #[inline]
    pub fn toggle(self) -> Self {
        match self {
            Lang::Uk => Lang::En,
            Lang::En => Lang::Uk,
        }
    }

    /// Button label for the language switcher in the top bar.
    #[inline]
    pub fn button_label(self) -> &'static str {
        match self {
            Lang::Uk => "[ UA | EN ]",
            Lang::En => "[ EN | UA ]",
        }
    }

    /// Current language indicator for display.
    #[allow(dead_code)]
    #[inline]
    pub fn current_label(self) -> &'static str {
        match self {
            Lang::Uk => "UA",
            Lang::En => "EN",
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

/// Toggles the global language and returns the new value.
#[inline]
pub fn toggle_lang() -> Lang {
    let new = current_lang().toggle();
    set_lang(new);
    new
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
    LangSwitcherTooltip,

    // Sidebar
    TabMemory,

    // Central panel (placeholder)
    AwaitingMemoryModel,

    // Self-test
    SelfTestPassedHeadless,
    SelfTestPassedLive,
    SelfTestFailed,

    // Memory module (future)
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

            (Lang::Uk, Key::LangSwitcherTooltip) => "Перемкнути мову інтерфейсу",
            (Lang::En, Key::LangSwitcherTooltip) => "Switch interface language",

            // Sidebar
            (Lang::Uk, Key::TabMemory) => "🧠 Пам'ять",
            (Lang::En, Key::TabMemory) => "🧠 Memory",

            // Central panel
            (Lang::Uk, Key::AwaitingMemoryModel) => {
                "Ядро телеметрії активне. Очікування моделі пам'яті (v0.0.4)..."
            }
            (Lang::En, Key::AwaitingMemoryModel) => {
                "Telemetry core active. Awaiting memory model (v0.0.4)..."
            }

            // Self-test
            (Lang::Uk, Key::SelfTestPassedHeadless) => "SELF-TEST PASSED (HEADLESS CI)",
            (Lang::En, Key::SelfTestPassedHeadless) => "SELF-TEST PASSED (HEADLESS CI)",

            (Lang::Uk, Key::SelfTestPassedLive) => "SELF-TEST PASSED (LIVE GUI)",
            (Lang::En, Key::SelfTestPassedLive) => "SELF-TEST PASSED (LIVE GUI)",

            (Lang::Uk, Key::SelfTestFailed) => "SELF-TEST FAILED",
            (Lang::En, Key::SelfTestFailed) => "SELF-TEST FAILED",

            // Memory module (future)
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
    fn test_lang_toggle() {
        assert_eq!(Lang::Uk.toggle(), Lang::En);
        assert_eq!(Lang::En.toggle(), Lang::Uk);
    }

    #[test]
    fn test_current_lang_get_set() {
        set_lang(Lang::En);
        assert_eq!(current_lang(), Lang::En);
        set_lang(Lang::Uk);
        assert_eq!(current_lang(), Lang::Uk);
    }

    #[test]
    fn test_toggle_lang() {
        set_lang(Lang::Uk);
        let new = toggle_lang();
        assert_eq!(new, Lang::En);
        assert_eq!(current_lang(), Lang::En);

        let new = toggle_lang();
        assert_eq!(new, Lang::Uk);
        assert_eq!(current_lang(), Lang::Uk);
    }

    #[test]
    fn test_key_translation_uk() {
        set_lang(Lang::Uk);
        assert_eq!(Key::AppTitle.t(), "⚡ PULSE // TELEMETRY");
        assert_eq!(Key::TabMemory.t(), "🧠 Пам'ять");
        assert_eq!(Key::UptimeLabel.t(), "Uptime");
        assert_eq!(
            Key::AwaitingMemoryModel.t(),
            "Ядро телеметрії активне. Очікування моделі пам'яті (v0.0.4)..."
        );
    }

    #[test]
    fn test_key_translation_en() {
        set_lang(Lang::En);
        assert_eq!(Key::AppTitle.t(), "⚡ PULSE // TELEMETRY");
        assert_eq!(Key::TabMemory.t(), "🧠 Memory");
        assert_eq!(Key::UptimeLabel.t(), "Uptime");
        assert_eq!(
            Key::AwaitingMemoryModel.t(),
            "Telemetry core active. Awaiting memory model (v0.0.4)..."
        );
    }

    #[test]
    fn test_button_labels() {
        assert_eq!(Lang::Uk.button_label(), "[ UA | EN ]");
        assert_eq!(Lang::En.button_label(), "[ EN | UA ]");
    }

    #[test]
    fn test_self_test_messages() {
        set_lang(Lang::En);
        assert_eq!(
            Key::SelfTestPassedHeadless.t(),
            "SELF-TEST PASSED (HEADLESS CI)"
        );
        assert_eq!(Key::SelfTestPassedLive.t(), "SELF-TEST PASSED (LIVE GUI)");
        assert_eq!(Key::SelfTestFailed.t(), "SELF-TEST FAILED");
    }
}

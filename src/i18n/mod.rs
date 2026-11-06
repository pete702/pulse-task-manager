//! Internationalization (i18n) for PULSE: detects the Windows UI language.
//! Core module with language detection, translation keys, and frame/window strings.
//! **100% English Code** — Ukrainian translations removed; locale detection retained for top-bar display.

use std::sync::atomic::{AtomicU8, Ordering};

/// Supported interface languages. Only English (En) is used for UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
}

impl Lang {
    #[allow(dead_code)]
    #[inline]
    pub fn current_label(self) -> &'static str {
        "EN"
    }
    #[allow(dead_code)]
    #[inline]
    pub fn locale_string(self) -> &'static str {
        "EN-US (0x0409)"
    }
    fn translate(self, key: Key) -> &'static str {
        if let Some(s) = self.translate_frame(key) {
            return s;
        }
        if let Some(s) = self.translate_memory(key) {
            return s;
        }
        ""
    }
}

static CURRENT_LANG: AtomicU8 = AtomicU8::new(Lang::En as u8);
#[cfg(test)]
thread_local! { static TEST_LANG: std::cell::Cell<Lang> = const { std::cell::Cell::new(Lang::En) }; }

/// Gets the current global language. Always returns En for UI; locale detection is separate.
#[inline]
pub fn current_lang() -> Lang {
    #[cfg(test)]
    {
        TEST_LANG.with(|lang| lang.get())
    }
    #[cfg(not(test))]
    {
        Lang::En
    }
}

/// Sets the global language (for testing only).
#[inline]
pub fn set_lang(lang: Lang) {
    CURRENT_LANG.store(lang as u8, Ordering::Relaxed);
    #[cfg(test)]
    TEST_LANG.with(|cell| cell.set(lang));
}

/// Initializes language from OS detection.
/// Called once at startup. Can be overridden by `--lang=en` CLI arg for testing.
/// Note: UI is always English; locale display uses GetUserDefaultLocaleName.
pub fn init_language(cli_lang_override: Option<&str>) {
    let lang = if let Some(override_str) = cli_lang_override {
        match override_str {
            "en" => Lang::En,
            _ => detect_os_language().0,
        }
    } else {
        detect_os_language().0
    };
    set_lang(lang);
}

/// UI language is always En; locale name is resolved in frame via GetUserDefaultLocaleName.
pub fn detect_os_language() -> (Lang, u16) {
    (Lang::En, 0x0409)
}

/// Returns the short OS locale badge for the top bar (e.g., "LOCALE: UK-UA").
#[allow(dead_code)]
pub fn os_locale_badge() -> &'static str {
    crate::i18n::frame::locale::os_locale_badge()
}

/// Returns the full OS locale string for hover tooltip.
#[allow(dead_code)]
pub fn os_locale_full() -> &'static str {
    crate::i18n::frame::locale::os_locale_full()
}

/// Bare short code for top bar (e.g., "UK-UA").
pub fn os_locale_short() -> &'static str {
    crate::i18n::frame::locale::os_locale_short()
}

pub use keys::Key;
impl Key {
    #[inline]
    pub fn t(self) -> &'static str {
        current_lang().translate(self)
    }
    #[allow(dead_code)]
    #[inline]
    pub fn t_lang(self, lang: Lang) -> &'static str {
        lang.translate(self)
    }
}

#[allow(unused_imports)]
pub use frame::{commit_limit_label, kernel_pool_detail, os_activity, system_cache_subtitle};

pub mod frame;
mod keys;
pub mod memory;
#[cfg(test)]
mod tests_basic;
#[cfg(test)]
mod tests_formatted;
#[cfg(test)]
mod tests_translations_en;

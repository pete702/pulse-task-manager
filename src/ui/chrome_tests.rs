//! UI Chrome tests.

use crate::i18n::{current_lang, os_locale_badge, set_lang, Lang};

#[test]
fn test_locale_strings() {
    set_lang(Lang::En);
    assert_eq!(current_lang(), Lang::En);
    let badge = os_locale_badge();
    assert!(!badge.is_empty());
    assert!(badge.starts_with("LOCALE:"));
}

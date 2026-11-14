//! Basic i18n tests: Lang enum, language switching, OS detection.

use crate::i18n::{current_lang, detect_os_language, init_language, set_lang, Lang};

#[test]
fn test_lang_default_is_en() {
    assert_eq!(Lang::default(), Lang::En);
}

#[test]
fn test_current_lang_get_set() {
    set_lang(Lang::En);
    assert_eq!(current_lang(), Lang::En);
    // Only En variant exists now
    assert_eq!(current_lang(), Lang::En);
}

#[test]
fn test_init_language_override() {
    init_language(Some("en"));
    assert_eq!(current_lang(), Lang::En);
    init_language(None); // Use OS detection
    assert_eq!(current_lang(), Lang::En);
}

#[test]
fn test_detect_os_language() {
    // This test just verifies the function exists and returns a valid tuple
    let (lang, lang_id) = detect_os_language();
    assert_eq!(lang, Lang::En);
    assert_ne!(lang_id, 0);
}

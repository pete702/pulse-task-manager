//! UI Chrome tests.

use crate::i18n::{current_lang, set_lang, Lang};

#[test]
fn test_locale_strings() {
    set_lang(Lang::En);
    assert_eq!(current_lang(), Lang::En);
    let short = crate::i18n::os_locale_short();
    assert!(!short.is_empty());
}

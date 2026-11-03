//! Formatted string tests for PULSE i18n.

use crate::i18n::{self, set_lang, Key, Lang};

#[test]
fn test_formatted_strings_en() {
    set_lang(Lang::En);
    assert_eq!(
        i18n::system_cache_subtitle(13.4),
        "OS Reserve • 13.4% of RAM"
    );
    assert_eq!(
        i18n::kernel_pool_detail("790.4 MiB", "455.0 MiB"),
        "Paged: 790.4 MiB / Non-paged: 455.0 MiB"
    );
    assert_eq!(
        i18n::commit_limit_label("30.5 GiB", 67.5),
        "Limit: 30.5 GiB (67.5%)"
    );
    assert_eq!(
        i18n::os_activity(181, 4318, 123_721),
        "181 processes | 4318 threads | 123721 handles"
    );
    assert_eq!(Key::OsActivityTitle.t(), "OS Activity");
}

#[test]
fn test_formatted_strings_have_no_placeholders() {
    set_lang(Lang::En);
    let text = format!(
        "{} {} {} {}",
        i18n::system_cache_subtitle(10.0),
        i18n::kernel_pool_detail("1 MiB", "2 MiB"),
        i18n::commit_limit_label("3 GiB", 4.0),
        i18n::os_activity(5, 6, 7)
    );
    assert!(!text.contains("{}"), "unformatted template leaked: {text}");
    assert!(!text.contains("{:"), "unformatted template leaked: {text}");
}

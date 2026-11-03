use std::fs;
use std::path::Path;

const MAX_MD_LINES: usize = 120;
const MAX_MD_BYTES: usize = 8_192;

const DOC_FILES: &[&str] = &["README.md", "ARCHITECTURE.md", "CHANGELOG.md"];

const FORBIDDEN_TOKENS: &[&str] = &[
    "../PULSE_",
    "..\\PULSE_",
    "PULSE_BUILDS",
    "PULSE_BACKUPS",
    "PULSE_LATEST",
    "SHA-256",
    "### Verified",
    "pulse-manager/pulse_manager",
    "0.9 MB",
    "4.4 MB",
    "0.37 ms",
    "100% Anti-Cheat",
    "free forever",
];

#[test]
fn markdown_docs_respect_size_and_content_rules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for rel in DOC_FILES {
        let path = root.join(rel);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        let line_count = content.lines().count();
        let byte_count = content.len();

        assert!(
            line_count <= MAX_MD_LINES,
            "{} has {} lines, exceeding limit of {}",
            rel,
            line_count,
            MAX_MD_LINES
        );
        assert!(
            byte_count <= MAX_MD_BYTES,
            "{} has {} bytes, exceeding limit of {}",
            rel,
            byte_count,
            MAX_MD_BYTES
        );

        for token in FORBIDDEN_TOKENS {
            assert!(
                !content.contains(token),
                "{} contains forbidden token: {:?}",
                rel,
                token
            );
        }

        // NUL byte check
        assert!(!content.contains('\0'), "{} contains NUL byte(s)", rel);
        // Cyrillic character check
        assert!(
            !content
                .chars()
                .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
            "{} contains Cyrillic characters",
            rel
        );
    }
}

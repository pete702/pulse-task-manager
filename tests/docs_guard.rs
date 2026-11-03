use std::fs;
use std::path::Path;

const MAX_MD_LINES: usize = 120;
const MAX_MD_BYTES: usize = 8_192;
const MAX_RS_LINES: usize = 165;

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

const SOURCE_FILES: &[&str] = &["Cargo.toml", "tests/docs_guard.rs"];

const TEXT_EXTS: &[&str] = &[
    "rs", "toml", "md", "yml", "yaml", "json", "txt", "rc", "manifest",
];
const SKIP_DIRS: &[&str] = &[".git", "target"];

fn is_text_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| TEXT_EXTS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| SKIP_DIRS.contains(&name))
        .unwrap_or(false)
}

fn walk_text_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
    if root.is_dir() {
        for entry in
            fs::read_dir(root).unwrap_or_else(|_| panic!("Failed to read dir: {}", root.display()))
        {
            let entry = entry.unwrap();
            let path = entry.path();
            if should_skip_dir(&path) {
                continue;
            }
            if path.is_dir() {
                walk_text_files(&path, files);
            } else if is_text_file(&path) {
                files.push(path);
            }
        }
    }
}

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
        // Cyrillic character check (U+0400..=U+052F)
        assert!(
            !content
                .chars()
                .any(|c| ('\u{0400}'..='\u{052F}').contains(&c)),
            "{} contains Cyrillic characters",
            rel
        );
    }
}

#[test]
fn source_files_have_no_nul_or_cyrillic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for rel in SOURCE_FILES {
        let path = root.join(rel);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        // NUL byte check
        assert!(!content.contains('\0'), "{} contains NUL byte(s)", rel);
        // Cyrillic character check (U+0400..=U+052F)
        assert!(
            !content
                .chars()
                .any(|c| ('\u{0400}'..='\u{052F}').contains(&c)),
            "{} contains Cyrillic characters",
            rel
        );
    }
}

#[test]
fn all_text_files_no_nul_or_cyrillic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    walk_text_files(root, &mut files);

    for path in &files {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();

        // NUL byte check
        assert!(!content.contains('\0'), "{} contains NUL byte(s)", rel);
        // Cyrillic character check (U+0400..=U+052F)
        assert!(
            !content
                .chars()
                .any(|c| ('\u{0400}'..='\u{052F}').contains(&c)),
            "{} contains Cyrillic characters",
            rel
        );
    }
}

#[test]
fn all_rs_files_under_src_under_line_limit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk_text_files(&root, &mut files);

    for path in &files {
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
            let line_count = content.lines().count();
            let rel = path
                .strip_prefix(root.parent().unwrap())
                .unwrap_or(path)
                .display()
                .to_string();
            assert!(
                line_count <= MAX_RS_LINES,
                "{} has {} lines, exceeding limit of {}",
                rel,
                line_count,
                MAX_RS_LINES
            );
        }
    }
}

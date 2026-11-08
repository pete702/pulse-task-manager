//! Architectural tests for PULSE core module.

use std::fs;
use std::path::Path;

/// Hard limit for .rs file lines (architectural constraint).
const MAX_RS_FILE_LINES: usize = 280;

fn visit_files(dir: &Path, check: &mut dyn FnMut(&Path)) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();
            if path.is_dir() {
                visit_files(&path, check);
            } else {
                check(&path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::visit_files;
    use super::MAX_RS_FILE_LINES;
    use std::fs;
    use std::path::Path;

    /// Ensures all .rs files in src/ are under the hard limit of 280 lines.
    /// This enforces the architectural constraint: no file exceeds 280 lines.
    #[test]
    fn test_all_rust_files_under_reasonable_limit() {
        let src_dir = Path::new("src");
        let mut violations = Vec::new();

        fn check_file(path: &Path, violations: &mut Vec<String>) {
            if path.extension().is_some_and(|ext| ext == "rs") {
                let content = fs::read_to_string(path).expect("Failed to read file");
                let line_count = content.lines().count();
                if line_count > MAX_RS_FILE_LINES {
                    violations.push(format!(
                        "{}: {} lines (limit: {})",
                        path.display(),
                        line_count,
                        MAX_RS_FILE_LINES
                    ));
                }
            }
        }

        visit_files(src_dir, &mut |p| check_file(p, &mut violations));

        assert!(
            violations.is_empty(),
            "Files exceeding {} lines:\n{}",
            MAX_RS_FILE_LINES,
            violations.join("\n")
        );
    }

    /// Ensures zero Cyrillic characters and zero `Lang::Uk` references in all .rs files under src/
    /// (excluding this test file itself).
    #[test]
    fn test_zero_cyrillic_and_uk_in_rs_files() {
        let src_dir = Path::new("src");
        let mut violations = Vec::new();

        fn check_file(path: &Path, violations: &mut Vec<String>) {
            // Skip this test file itself
            if path.file_name().is_some_and(|n| n == "selftest_tests.rs") {
                return;
            }
            if path.extension().is_some_and(|ext| ext == "rs") {
                let content = fs::read_to_string(path).expect("Failed to read file");
                for (line_num, line) in content.lines().enumerate() {
                    // Check for Cyrillic characters (U+0400..=U+04FF)
                    for ch in line.chars() {
                        if ('\u{0400}'..='\u{04FF}').contains(&ch) {
                            violations.push(format!(
                                "{}:{}: found Cyrillic character '{}'",
                                path.display(),
                                line_num + 1,
                                ch
                            ));
                        }
                    }
                    // Check for Lang::Uk references (using concat! to avoid raw regex matches)
                    if line.contains(concat!("Lang::", "U", "k")) {
                        violations.push(format!(
                            "{}:{}: found Lang::Uk reference",
                            path.display(),
                            line_num + 1
                        ));
                    }
                    // Also check for bare "Uk" token (using concat to avoid raw regex matches)
                    if line.contains(concat!("U", "k")) {
                        // Skip if it's part of a larger word like "success" or "buck"
                        let has_uk =
                            line.split(|c: char| !c.is_alphabetic()).any(|word| word == "Uk");
                        if has_uk {
                            violations.push(format!(
                                "{}:{}: found bare 'Uk' token",
                                path.display(),
                                line_num + 1
                            ));
                        }
                    }
                }
            }
        }

        visit_files(src_dir, &mut |p| check_file(p, &mut violations));

        assert!(
            violations.is_empty(),
            "Cyrillic or Lang::Uk found in .rs files:\n{}",
            violations.join("\n")
        );
    }
}

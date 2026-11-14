//! Protected process name lists and path-aware safety categories.

/// Kernel pseudo-processes (protected only when identity/path indicates system context).
/// Entries are canonical normalized names (lowercase basename, no ".exe" suffix).
pub(crate) const KERNEL_PSEUDO_PROCESSES: &[&str] =
    &["system", "registry", "memory compression", "memcompression", "secure system", "idle"];

/// Windows OS system binaries (protected only when under SystemRoot).
/// Entries are canonical normalized names (lowercase basename, no ".exe" suffix).
pub(crate) const WINDOWS_OS_SYSTEM_BINARIES: &[&str] = &[
    "smss",
    "csrss",
    "wininit",
    "services",
    "lsass",
    "lsaiso",
    "svchost",
    "dwm",
    "winlogon",
    "fontdrvhost",
    "sihost",
    "taskhostw",
    "explorer",
    "ctfmon",
    "smartscreen",
    "runtimebroker",
    "searchprotocolhost",
    "searchfilterhost",
    "spoolsv",
    "audiodg",
    "wmiprvse",
    "searchindexer",
    "startmenuexperiencehost",
    "searchhost",
    "shellexperiencehost",
    "textinputhost",
    "conhost",
];

/// Security/Defender/anti-cheat user-mode binaries (protected only in trusted roots).
/// Entries are canonical normalized names (lowercase basename, no ".exe" suffix).
pub(crate) const SECURITY_DEFENDER_ANTICHEAT: &[&str] = &[
    "msmpeng",
    "nissrv",
    "securityhealthservice",
    "securityhealthsystray",
    "mpcmdrun",
    "vgc",
    "vgtray",
    "easyanticheat",
    "easyanticheat_eos",
    "beservice",
    "zksvc",
    "faceitclient",
    "eseaclient",
    "ricochet",
];

/// Canonical normalization: basename, trim, ASCII lowercase, strip trailing ".exe".
pub(crate) fn normalize_process_name(name: &str) -> String {
    let base = name.rsplit(['\\', '/']).next().unwrap_or(name).trim();
    let lower = base.to_ascii_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

pub(crate) fn is_kernel_pseudo(norm: &str) -> bool {
    KERNEL_PSEUDO_PROCESSES.contains(&norm)
}
pub(crate) fn is_windows_os_binary(norm: &str) -> bool {
    WINDOWS_OS_SYSTEM_BINARIES.contains(&norm)
}
pub(crate) fn is_security_defender_anticheat(norm: &str) -> bool {
    SECURITY_DEFENDER_ANTICHEAT.contains(&norm)
}
pub(crate) fn is_any_protected_name(norm: &str) -> bool {
    is_kernel_pseudo(norm) || is_windows_os_binary(norm) || is_security_defender_anticheat(norm)
}
pub(crate) fn protection_reason(norm: &str) -> &'static str {
    if is_security_defender_anticheat(norm) {
        "Anti-cheat protected process skipped"
    } else {
        "Critical Windows OS process is protected"
    }
}

#[cfg(any(target_os = "windows", test))]
fn trim_trailing_seps(mut s: &str) -> &str {
    while s.len() > 3 && (s.ends_with('\\') || s.ends_with('/')) {
        s = &s[..s.len() - 1];
    }
    let b = s.as_bytes();
    if s.len() == 3 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/') {
        return s;
    }
    while s.ends_with('\\') || s.ends_with('/') {
        s = &s[..s.len() - 1];
    }
    s
}

#[cfg(target_os = "windows")]
fn env_dir(name: &str) -> Option<String> {
    std::env::var_os(name).and_then(|v| v.into_string().ok()).filter(|v| !v.trim().is_empty())
}
#[cfg(target_os = "windows")]
fn system_root_string() -> String {
    env_dir("SystemRoot").or_else(|| env_dir("windir")).unwrap_or_else(|| r"C:\Windows".to_string())
}

/// Normalizes a Win32 path for case-insensitive, component-aware comparison/matching.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn normalize_path_for_match(raw: &str) -> String {
    let mut s = raw.trim().replace('/', "\\");
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        s = format!(r"\\{}", rest);
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        s = rest.to_string();
    }
    #[cfg(target_os = "windows")]
    {
        let lower = s.to_ascii_lowercase();
        if lower == r"\systemroot" {
            s = system_root_string();
        } else if lower.starts_with(r"\systemroot\") {
            let orig_tail = &s[r"\systemroot".len()..];
            s = format!("{}{}", trim_trailing_seps(&system_root_string()), orig_tail);
        }
    }
    trim_trailing_seps(&s).to_ascii_lowercase()
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn is_path_under_dir(path_norm: &str, root_norm: &str) -> bool {
    let root = trim_trailing_seps(root_norm);
    if path_norm == root {
        return true;
    }
    path_norm.starts_with(&format!(r"{}\", root))
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn full_paths_match(a: &str, b: &str) -> bool {
    let na = normalize_path_for_match(a);
    let nb = normalize_path_for_match(b);
    !na.is_empty() && na == nb
}

#[cfg(target_os = "windows")]
pub(crate) fn is_under_system_root(path: &str) -> bool {
    let p = normalize_path_for_match(path);
    is_path_under_dir(&p, &normalize_path_for_match(&system_root_string()))
}
#[cfg(not(target_os = "windows"))]
pub(crate) fn is_under_system_root(_path: &str) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub(crate) fn is_under_security_trusted_root(path: &str) -> bool {
    let p = normalize_path_for_match(path);
    let mut roots = vec![normalize_path_for_match(&system_root_string())];
    if let Some(v) = env_dir("ProgramFiles") {
        roots.push(normalize_path_for_match(&v));
    }
    if let Some(v) = env_dir("ProgramFiles(x86)").or_else(|| env_dir("ProgramFiles(X86)")) {
        roots.push(normalize_path_for_match(&v));
    }
    if let Some(pd) = env_dir("ProgramData") {
        roots.push(normalize_path_for_match(&format!(
            r"{}\Microsoft\Windows Defender",
            trim_trailing_seps(&pd)
        )));
    } else {
        roots.push(normalize_path_for_match(r"C:\ProgramData\Microsoft\Windows Defender"));
    }
    roots.iter().any(|r| is_path_under_dir(&p, r))
}
#[cfg(not(target_os = "windows"))]
pub(crate) fn is_under_security_trusted_root(_path: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_self_match(slice: &[&str]) {
        for &entry in slice {
            assert_eq!(entry, normalize_process_name(entry), "entry not canonical: {entry}");
            assert!(is_any_protected_name(entry), "bare missing: {entry}");
            let with_exe = format!("{entry}.exe");
            assert!(
                is_any_protected_name(&normalize_process_name(&with_exe)),
                ".exe missing: {entry}"
            );
            let upper = format!("{}.EXE", entry.to_ascii_uppercase());
            assert!(
                is_any_protected_name(&normalize_process_name(&upper)),
                "upper missing: {entry}"
            );
            let path = format!(r"C:\Windows\System32\{}.exe", entry);
            assert!(is_any_protected_name(&normalize_process_name(&path)), "path missing: {entry}");
        }
    }

    #[test]
    fn test_protected_lists_self_match_and_required_entries() {
        assert_self_match(KERNEL_PSEUDO_PROCESSES);
        assert_self_match(WINDOWS_OS_SYSTEM_BINARIES);
        assert_self_match(SECURITY_DEFENDER_ANTICHEAT);
        for req in ["ctfmon", "smartscreen", "mpcmdrun"] {
            let with_exe = format!("{req}.exe");
            assert!(is_any_protected_name(&normalize_process_name(&with_exe)), "{req}");
        }
        for dead in ["bedaisy.sys", "hoyokprotect.sys", "bedaisy", "hoyokprotect"] {
            assert!(
                !is_any_protected_name(&normalize_process_name(dead)),
                "dead entry still present: {dead}"
            );
        }
    }
}

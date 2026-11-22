//! Manual Pure ASCII CSV/JSON export for lag receipts (no serde dependency).
use std::path::PathBuf;

use super::blackbox::BlackboxSample;

const CSV_HEADER: &str =
    "timestamp,cpu_pct,peak_core_pct,ram_pct,commit_pct,disk_mb_s,ping_ms,jitter_ms,culprit_name,verdict\n";

fn num(v: f32) -> String {
    if v.is_finite() {
        format!("{:.2}", v)
    } else {
        "0.00".to_string()
    }
}

fn csv_field(v: &str) -> String {
    let clean: String = v.chars().map(|c| if c.is_ascii() { c } else { '?' }).collect();
    if clean.contains(',') || clean.contains('"') || clean.contains('\n') || clean.contains('\r') {
        format!("\"{}\"", clean.replace('"', "\"\""))
    } else {
        clean
    }
}

fn json_str(v: &str) -> String {
    let mut out = String::with_capacity(v.len() + 2);
    out.push('"');
    for c in v.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii() => out.push(c),
            _ => out.push('?'),
        }
    }
    out.push('"');
    out
}

fn json_optional(v: &Option<String>) -> String {
    match v {
        Some(s) => json_str(s),
        None => "null".to_string(),
    }
}

/// Formats the blackbox window as a CSV document with a header row.
pub fn format_blackbox_csv(samples: &[BlackboxSample]) -> String {
    let mut out = String::from(CSV_HEADER);
    for s in samples {
        let verdict = s.verdict.as_deref().unwrap_or("");
        let row = [
            csv_field(&s.timestamp),
            num(s.cpu_pct),
            num(s.peak_core_pct),
            num(s.ram_pct),
            num(s.commit_pct),
            num(s.disk_mb_s),
            num(s.ping_ms),
            num(s.jitter_ms),
            csv_field(&s.culprit_name),
            csv_field(verdict),
        ];
        out.push_str(&row.join(","));
        out.push('\n');
    }
    out
}

fn json_sample(s: &BlackboxSample) -> String {
    let mut out = String::from("    {\n");
    out.push_str(&format!("      \"timestamp\": {},\n", json_str(&s.timestamp)));
    out.push_str(&format!("      \"cpu_pct\": {},\n", num(s.cpu_pct)));
    out.push_str(&format!("      \"peak_core_pct\": {},\n", num(s.peak_core_pct)));
    out.push_str(&format!("      \"ram_pct\": {},\n", num(s.ram_pct)));
    out.push_str(&format!("      \"commit_pct\": {},\n", num(s.commit_pct)));
    out.push_str(&format!("      \"disk_mb_s\": {},\n", num(s.disk_mb_s)));
    out.push_str(&format!("      \"ping_ms\": {},\n", num(s.ping_ms)));
    out.push_str(&format!("      \"jitter_ms\": {},\n", num(s.jitter_ms)));
    out.push_str(&format!("      \"culprit_name\": {},\n", json_str(&s.culprit_name)));
    out.push_str(&format!("      \"verdict\": {}\n", json_optional(&s.verdict)));
    out.push_str("    }");
    out
}

/// Formats the blackbox window as a minified-but-readable JSON document.
pub fn format_blackbox_json(samples: &[BlackboxSample]) -> String {
    let mut out = String::from("{\n  \"version\": 1,\n  \"window_secs\": 60,\n  \"samples\": [\n");
    for (idx, s) in samples.iter().enumerate() {
        out.push_str(&json_sample(s));
        if idx + 1 < samples.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn receipt_dir() -> Result<PathBuf, String> {
    if let Some(path) = super::persist::state_path() {
        if let Some(parent) = path.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    if super::persist::is_portable_mode() {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        return Ok(exe.parent().map(PathBuf::from).unwrap_or(exe));
    }
    std::env::var_os("LOCALAPPDATA")
        .map(|p| PathBuf::from(p).join("PULSE"))
        .ok_or_else(|| "no config directory available".to_string())
}

/// Exports the last minute as both JSON and CSV; returns the JSON path.
pub fn export_lag_receipt() -> Result<PathBuf, String> {
    let samples = super::blackbox::snapshot_samples();
    let dir = receipt_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let csv_path = dir.join("pulse_lag_receipt.csv");
    let json_path = dir.join("pulse_lag_receipt.json");
    std::fs::write(&csv_path, format_blackbox_csv(&samples)).map_err(|e| e.to_string())?;
    std::fs::write(&json_path, format_blackbox_json(&samples)).map_err(|e| e.to_string())?;
    Ok(json_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blackbox::BLACKBOX_CAPACITY;

    fn sample(i: usize) -> BlackboxSample {
        BlackboxSample {
            seq: i as u64,
            timestamp: format!("12:00:{:02}", i),
            cpu_pct: 10.0,
            peak_core_pct: 96.0,
            ram_pct: 40.0,
            commit_pct: 50.0,
            disk_mb_s: 5.0,
            ping_ms: 12.0,
            jitter_ms: 3.0,
            culprit_name: "game.exe".to_string(),
            verdict: Some(format!("12:00:{:02} | 1-Core 96%: game.exe", i)),
        }
    }

    #[test]
    fn csv_has_header_and_escapes_commas() {
        let mut s = sample(1);
        s.culprit_name = "a,b\"c".to_string();
        let csv = format_blackbox_csv(&[s]);
        assert!(csv.starts_with(CSV_HEADER));
        assert!(csv.contains("\"a,b\"\"c\""));
        assert_eq!(csv.lines().count(), 2);
    }

    #[test]
    fn csv_is_ascii_just_like_json() {
        let mut s = sample(3);
        s.culprit_name = "caf\u{e9}.exe,\u{2603}".to_string();
        let csv = format_blackbox_csv(&[s]);
        assert!(csv.is_ascii(), "CSV must be Pure ASCII");
        assert!(csv.contains("caf?.exe,?"));
    }

    #[test]
    fn json_is_ascii_with_all_keys_and_null_verdict() {
        let mut s = sample(2);
        s.verdict = None;
        let json = format_blackbox_json(&[s]);
        assert!(json.is_ascii());
        assert!(json.contains("\"window_secs\": 60"));
        assert!(json.contains("\"verdict\": null"));
        assert!(json.contains("\"cpu_pct\": 10.00"));
        assert!(json.contains("\"culprit_name\": \"game.exe\""));
        assert_eq!(BLACKBOX_CAPACITY, 60);
    }
}

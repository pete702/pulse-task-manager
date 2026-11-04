//! Shared formatters for the memory module.

/// Formats bytes into a compact binary unit string (GiB, MiB, KiB, B).
pub fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1_073_741_824;
    const MIB: u64 = 1_048_576;
    const KIB: u64 = 1024;

    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Formats bytes-per-second delta into a compact string with sign.
pub fn format_delta(delta: i64) -> String {
    let sign = if delta >= 0 { "+" } else { "-" };
    let abs = delta.unsigned_abs();
    format!("{} {}/s", sign, format_bytes(abs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1_073_741_824), "1.00 GiB");
        assert_eq!(format_bytes(1_048_576), "1.00 MiB");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(2_147_483_648), "2.00 GiB");
        assert_eq!(format_bytes(5_368_709_120), "5.00 GiB");
    }

    #[test]
    fn test_format_delta() {
        assert_eq!(format_delta(0), "+ 0 B/s");
        assert_eq!(format_delta(1024), "+ 1.00 KiB/s");
        assert_eq!(format_delta(-1024), "- 1.00 KiB/s");
        assert_eq!(format_delta(1_048_576), "+ 1.00 MiB/s");
    }
}

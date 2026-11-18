//! CPU table group aggregation helpers.

use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::cpu::model::ProcessCpuItem;

pub fn sanitize_group_cpu_percent(v: f32) -> f32 {
    if !v.is_finite() || v < 0.0 {
        0.0
    } else {
        v
    }
}

/// Formats the member count for a grouped clone row, e.g. "(3 procs)".
pub fn group_member_count_label(count: usize) -> String {
    format!("({count} procs)")
}

pub fn aggregate_grouped_items(items: Vec<ProcessCpuItem>) -> Vec<ProcessCpuItem> {
    let mut grouped: HashMap<Arc<str>, ProcessCpuItem> = HashMap::new();
    for mut item in items {
        item.cpu_pct = sanitize_group_cpu_percent(item.cpu_pct);
        let key: Arc<str> = Arc::clone(&item.name).to_ascii_lowercase().into();
        grouped
            .entry(key)
            .and_modify(|ex| {
                let add_cpu = sanitize_group_cpu_percent(item.cpu_pct).min(100.0);
                ex.cpu_pct = (sanitize_group_cpu_percent(ex.cpu_pct) + add_cpu).clamp(0.0, 100.0);
                ex.is_cpu_hog |= add_cpu >= crate::modules::cpu::model::CPU_HOG_THRESHOLD_PCT;
                ex.ram_bytes = ex.ram_bytes.saturating_add(item.ram_bytes);
                ex.disk_read_mb_s += item.disk_read_mb_s;
                ex.disk_write_mb_s += item.disk_write_mb_s;
                ex.ram_delta_10s = ex.ram_delta_10s.saturating_add(item.ram_delta_10s);
                ex.clone_count += 1;
                ex.update_formatted();
            })
            .or_insert(item);
    }
    let mut out: Vec<ProcessCpuItem> = grouped.into_values().collect();
    out.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.pid.cmp(&b.pid))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::group_member_count_label;

    #[test]
    fn group_member_count_label_formats() {
        assert_eq!(group_member_count_label(3), "(3 procs)");
        assert_eq!(group_member_count_label(1), "(1 procs)");
    }
}

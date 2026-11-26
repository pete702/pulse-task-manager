//! Cached badge text helpers and tab badge painting for the sidebar.
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

const BADGE_CACHE_CAP: usize = 256;

type BadgeCache = (HashMap<(u8, u16), Arc<str>>, VecDeque<(u8, u16)>);

#[allow(clippy::type_complexity)]
static BADGE_CACHE: OnceLock<Mutex<BadgeCache>> = OnceLock::new();

fn badge_cache() -> &'static Mutex<BadgeCache> {
    BADGE_CACHE.get_or_init(|| Mutex::new((HashMap::new(), VecDeque::new())))
}

fn cached_formatted(kind: u8, key: u16, make: impl FnOnce() -> String) -> Arc<str> {
    let k = (kind, key);
    {
        let cache = badge_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(value) = cache.0.get(&k) {
            return Arc::clone(value);
        }
    }
    let value = Arc::from(make());
    let mut cache = badge_cache().lock().unwrap_or_else(|e| e.into_inner());
    let (map, order) = &mut *cache;
    crate::util::insert_capped_fifo(map, order, BADGE_CACHE_CAP, k, Arc::clone(&value));
    value
}

pub fn cached_pct(pct: f32) -> Arc<str> {
    let pct = if pct.is_finite() { pct.clamp(0.0, 100.0) } else { 0.0 };
    let key = (pct * 10.0).round() as u16;
    cached_formatted(0, key, || format!("{}%", pct.round() as u32))
}

pub fn cached_cpu_pct(pct: f32) -> Arc<str> {
    let pct = if pct.is_finite() { pct.clamp(0.0, 100.0) } else { 0.0 };
    let key = (pct * 10.0).round() as u16;
    cached_formatted(1, key, || format!("{}%", pct.round() as u32))
}

pub fn cached_network_badge(total_mbps: f32) -> Arc<str> {
    let total = if total_mbps.is_finite() { total_mbps.max(0.0) } else { 0.0 };
    let key = (total * 10.0).round().min(u16::MAX as f32) as u16;
    cached_formatted(2, key, || {
        if total >= 1000.0 {
            format!("{:.2} Gbps", total / 1000.0)
        } else {
            format!("{:.1} Mbps", total)
        }
    })
}

pub fn cached_count(count: u32) -> Arc<str> {
    let key = count.min(u16::MAX as u32) as u16;
    cached_formatted(3, key, || count.to_string())
}

pub fn cached_storage_badge(total_mb_s: f32) -> Arc<str> {
    let total = if total_mb_s.is_finite() { total_mb_s.max(0.0) } else { 0.0 };
    let key = (total * 10.0).round().min(u16::MAX as f32) as u16;
    cached_formatted(4, key, || format!("{:.1} MB/s", total))
}

pub fn badge_active(total: f32, state: &mut bool) -> bool {
    let total = if total.is_finite() { total.max(0.0) } else { 0.0 };
    if *state {
        if total < 0.05 {
            *state = false;
        }
    } else if total > 0.15 {
        *state = true;
    }
    *state
}

pub(super) fn hysteresis_active(slot: &Mutex<(u64, bool)>, scope: u64, total: f32) -> bool {
    let total = if total.is_finite() { total.max(0.0) } else { 0.0 };
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if guard.0 != scope {
        guard.0 = scope;
        guard.1 = false;
    }
    let mut cur = guard.1;
    let next = badge_active(total, &mut cur);
    guard.1 = next;
    next
}

pub(super) fn draw_badge_text(
    ui: &eframe::egui::Ui,
    rect: eframe::egui::Rect,
    text: &str,
    size: f32,
    color: eframe::egui::Color32,
) {
    use crate::theme::colors;
    let w = 64.0_f32.min(rect.width() * 0.42);
    let pill = eframe::egui::Rect::from_min_max(
        eframe::egui::pos2(rect.right() - 10.0 - w, rect.center().y - 9.0),
        eframe::egui::pos2(rect.right() - 10.0, rect.center().y + 9.0),
    );
    let painter = ui.painter();
    let radius = eframe::egui::Rounding::same(colors::BADGE_RADIUS);
    painter.rect_filled(pill, radius, colors::ELEVATED);
    painter.rect_stroke(pill, radius, eframe::egui::Stroke::new(1.0_f32, colors::BORDER));
    painter.text(
        pill.center(),
        eframe::egui::Align2::CENTER_CENTER,
        text,
        eframe::egui::FontId::monospace(size),
        color,
    );
}

/// Badge text matches the GPU % sparkline: same metric, honest `--` when missing.
fn system_badge_text() -> String {
    let tail = crate::ui::system_gpu_tab::system_gpu_chart::recent_gpu_series(1);
    match tail.first().copied().flatten() {
        Some(v) if v.is_finite() => format!("GPU {}%", v.round() as u32),
        _ => "GPU --".to_owned(),
    }
}

/// Live GPU engine badge for tab [05]; never the old "v0.5" placeholder.
pub(crate) fn draw_system_badge(ui: &eframe::egui::Ui, rect: eframe::egui::Rect) {
    let text = system_badge_text();
    draw_badge_text(ui, rect, &text, 12.0, crate::theme::colors::ACCENT_CYAN);
}

pub(super) fn draw_tab_badge(
    ui: &eframe::egui::Ui,
    rect: eframe::egui::Rect,
    tab: crate::app_tabs::ActiveTab,
    snap: &crate::core::SystemSnapshot,
    storage_latch: &Mutex<(u64, bool)>,
    network_latch: &Mutex<(u64, bool)>,
) {
    if rect.width() < 140.0 {
        return;
    }
    use crate::app_tabs::ActiveTab as T;
    match tab {
        T::Processes => {
            let cpu_n = snap.cpu.as_ref().map(|c| c.process_count).unwrap_or(0);
            let mem_n = snap.memory.as_ref().map(|m| m.processes.len()).unwrap_or(0);
            let n = cpu_n.max(mem_n) as u32;
            draw_badge_text(ui, rect, &cached_count(n), 14.0, crate::theme::colors::ACCENT_CYAN);
        }
        T::Memory => {
            if let Some(ref m) = snap.memory {
                let pct = m.ram_usage_percent();
                let pct = if pct.is_finite() { pct } else { 0.0 };
                draw_badge_text(ui, rect, &cached_pct(pct), 14.0, crate::theme::load_color(pct));
            }
        }
        T::Cpu => {
            if let Some(ref c) = snap.cpu {
                let pct = c.total_pct;
                let pct = if pct.is_finite() { pct } else { 0.0 };
                draw_badge_text(
                    ui,
                    rect,
                    &cached_cpu_pct(pct),
                    14.0,
                    crate::theme::load_color(pct),
                );
            }
        }
        T::Storage => {
            let t = snap.storage.read_mb_s + snap.storage.write_mb_s;
            let t = if t.is_finite() { t.max(0.0) } else { 0.0 };
            let vols = &snap.storage.volumes;
            let scope = vols
                .iter()
                .fold(0u64, |a, v| a.wrapping_mul(31).wrapping_add(v.mount_point.len() as u64));
            let col = if hysteresis_active(storage_latch, scope, t) {
                crate::theme::colors::ACCENT_CYAN
            } else {
                crate::theme::colors::TEXT_SECONDARY
            };
            draw_badge_text(ui, rect, &cached_storage_badge(t), 14.0, col);
        }
        T::Network => {
            let t = snap.network.rx_mbps + snap.network.tx_mbps;
            let t = if t.is_finite() { t.max(0.0) } else { 0.0 };
            let adapters = &snap.network.adapters;
            let scope = adapters
                .iter()
                .fold(0u64, |a, v| a.wrapping_mul(31).wrapping_add(v.name.len() as u64));
            let col = if hysteresis_active(network_latch, scope, t) {
                crate::theme::colors::ACCENT_CYAN
            } else {
                crate::theme::colors::TEXT_SECONDARY
            };
            draw_badge_text(ui, rect, &cached_network_badge(t), 14.0, col);
        }
        T::System => draw_system_badge(ui, rect),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn badge_non_finite_fallback() {
        assert_eq!(&*cached_pct(f32::NAN), "0%");
        assert_eq!(&*cached_pct(f32::INFINITY), "0%");
        assert_eq!(&*cached_cpu_pct(f32::NAN), "0%");
        assert_eq!(&*cached_cpu_pct(f32::INFINITY), "0%");
        assert_eq!(&*cached_storage_badge(f32::NAN), "0.0 MB/s");
        assert_eq!(&*cached_network_badge(f32::INFINITY), "0.0 Mbps");
        let mut st = false;
        assert!(!badge_active(f32::NAN, &mut st));
        assert!(!badge_active(f32::INFINITY, &mut st));
    }

    #[test]
    fn system_badge_text_always_labels_gpu_metric() {
        assert!(system_badge_text().starts_with("GPU"));
    }

    #[test]
    fn badge_cache_fifo_evicts_oldest_only() {
        use std::collections::{HashMap, VecDeque};
        let mut map: HashMap<(u8, u16), Arc<str>> = HashMap::new();
        let mut order: VecDeque<(u8, u16)> = VecDeque::new();
        for i in 0..BADGE_CACHE_CAP + 1 {
            let key = (3u8, i as u16);
            crate::util::insert_capped_fifo(
                &mut map,
                &mut order,
                BADGE_CACHE_CAP,
                key,
                Arc::from(i.to_string()),
            );
        }
        assert_eq!(map.len(), BADGE_CACHE_CAP);
        assert!(!map.contains_key(&(3, 0)));
        assert!(map.contains_key(&(3, 1)));
        assert!(map.contains_key(&(3, BADGE_CACHE_CAP as u16)));
    }
}

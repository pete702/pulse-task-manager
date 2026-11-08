//! Cached badge text helpers and tab badge painting for the sidebar.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
pub fn cached_pct(pct: f32) -> Arc<str> {
    let pct = if pct.is_finite() { pct.clamp(0.0, 100.0) } else { 0.0 };
    let key = (pct * 10.0).round() as u16;
    static C: OnceLock<Mutex<(u16, Arc<str>)>> = OnceLock::new();
    let mut cache = C
        .get_or_init(|| Mutex::new((u16::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{}%", pct.round() as u32));
    }
    Arc::clone(&cache.1)
}
pub fn cached_cpu_pct(pct: f32) -> Arc<str> {
    let pct = if pct.is_finite() { pct.clamp(0.0, 100.0) } else { 0.0 };
    let key = (pct * 10.0).round() as u16;
    static C: OnceLock<Mutex<(u16, Arc<str>)>> = OnceLock::new();
    let mut cache = C
        .get_or_init(|| Mutex::new((u16::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{}%", pct.round() as u32));
    }
    Arc::clone(&cache.1)
}
pub fn cached_network_badge(total_mbps: f32) -> Arc<str> {
    let total = if total_mbps.is_finite() { total_mbps.max(0.0) } else { 0.0 };
    let key = (total * 10.0).round() as u64;
    static C: OnceLock<Mutex<(u64, Arc<str>)>> = OnceLock::new();
    let mut cache = C
        .get_or_init(|| Mutex::new((u64::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = if total >= 1000.0 {
            Arc::from(format!("{:.2} Gbps", total / 1000.0))
        } else {
            Arc::from(format!("{:.1} Mbps", total))
        };
    }
    Arc::clone(&cache.1)
}
pub fn cached_count(count: u32) -> Arc<str> {
    static C: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = C
        .get_or_init(|| Mutex::new((u32::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != count {
        cache.0 = count;
        cache.1 = Arc::from(count.to_string());
    }
    Arc::clone(&cache.1)
}
pub fn cached_storage_badge(total_mb_s: f32) -> Arc<str> {
    let total = if total_mb_s.is_finite() { total_mb_s.max(0.0) } else { 0.0 };
    let key = (total * 10.0).round() as u32;
    static C: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = C
        .get_or_init(|| Mutex::new((u32::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{:.1} MB/s", total));
    }
    Arc::clone(&cache.1)
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
pub(super) fn hysteresis_active(slot: &AtomicBool, total: f32) -> bool {
    let total = if total.is_finite() { total.max(0.0) } else { 0.0 };
    let mut cur = slot.load(Ordering::Relaxed);
    let next = badge_active(total, &mut cur);
    slot.store(next, Ordering::Relaxed);
    next
}
pub(super) fn draw_badge_text(
    ui: &eframe::egui::Ui,
    rect: eframe::egui::Rect,
    text: &str,
    size: f32,
    color: eframe::egui::Color32,
) {
    let pos = eframe::egui::pos2(rect.right() - 12.0, rect.center().y);
    ui.painter().text(
        pos,
        eframe::egui::Align2::RIGHT_CENTER,
        text,
        eframe::egui::FontId::monospace(size),
        color,
    );
}
pub(super) fn draw_tab_badge(
    ui: &eframe::egui::Ui,
    rect: eframe::egui::Rect,
    tab: crate::app_tabs::ActiveTab,
    snap: &crate::core::SystemSnapshot,
    storage_active: &AtomicBool,
    network_active: &AtomicBool,
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
            let col = if hysteresis_active(storage_active, t) {
                crate::theme::colors::ACCENT_CYAN
            } else {
                crate::theme::colors::TEXT_SECONDARY
            };
            draw_badge_text(ui, rect, &cached_storage_badge(t), 14.0, col);
        }
        T::Network => {
            let t = snap.network.rx_mbps + snap.network.tx_mbps;
            let t = if t.is_finite() { t.max(0.0) } else { 0.0 };
            let col = if hysteresis_active(network_active, t) {
                crate::theme::colors::ACCENT_CYAN
            } else {
                crate::theme::colors::TEXT_SECONDARY
            };
            draw_badge_text(ui, rect, &cached_network_badge(t), 14.0, col);
        }
        T::System => draw_badge_text(
            ui,
            rect,
            "v0.5",
            11.0,
            eframe::egui::Color32::from_rgba_unmultiplied(92, 104, 122, 190),
        ),
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
}

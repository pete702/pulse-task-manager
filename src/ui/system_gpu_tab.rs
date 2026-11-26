//! Live [05] System & GPU dashboard: compact bar + 2-column cockpit + consumers.
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::app_cmd::AppCommand;
use crate::core::SystemSnapshot;
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::cpu::table_state::CpuTableState;
use crate::platform::dxgi_gpu::DxgiGpuAdapter;
use crate::platform::pdh_gpu::GpuTelemetrySnapshot;
use crate::theme::{card_frame, colors, OPTIMAL_EMERALD};
use crate::ui::system_services_view::{
    render_compact_bar, render_services_view, render_startup_view, SystemViewMode,
};

#[path = "system_gpu_chart.rs"]
pub mod system_gpu_chart;
#[path = "system_gpu_chart_hud.rs"]
pub mod system_gpu_chart_hud;
#[path = "system_gpu_table.rs"]
pub mod system_gpu_table;

pub use system_gpu_chart::gpu_badge;

const REFRESH: Duration = Duration::from_secs(2);
const UPPER_H: f32 = 265.0;
const MIB: f64 = 1_048_576.0;
struct GpuViewCache {
    last_refresh: Instant,
    adapters: Vec<DxgiGpuAdapter>,
}
static CACHE: OnceLock<Mutex<GpuViewCache>> = OnceLock::new();

fn cache() -> &'static Mutex<GpuViewCache> {
    CACHE.get_or_init(|| {
        Mutex::new(GpuViewCache { last_refresh: Instant::now() - REFRESH, adapters: Vec::new() })
    })
}

fn refresh_adapters_if_stale() {
    let mut guard = cache().lock().unwrap_or_else(|e| e.into_inner());
    if guard.last_refresh.elapsed() >= REFRESH {
        guard.adapters = crate::platform::dxgi_gpu::query_dxgi_gpus();
        guard.last_refresh = Instant::now();
    }
}

/// DXGI limits for the ingest-side VRAM percentage, refreshed at most every 2 s.
pub fn background_adapters() -> Vec<DxgiGpuAdapter> {
    refresh_adapters_if_stale();
    cache().lock().unwrap_or_else(|e| e.into_inner()).adapters.clone()
}

/// Uppercase `Vendor:Device` PCI identifier, e.g. `PCI 10DE:2882`.
fn pci_id(vendor: u32, device: u32) -> String {
    format!("PCI {:04X}:{:04X}", vendor, device)
}

/// One non-red classification pill for the adapter card.
fn pill_label(integrated: bool) -> (&'static str, egui::Color32) {
    if integrated {
        ("Integrated", OPTIMAL_EMERALD)
    } else {
        ("Discrete GPU", colors::ACCENT_CYAN)
    }
}

/// Rectangular bar with a >= 4 px fill whenever usage is non-zero.
/// A zero dedicated limit renders only the `-- / --` caption, never a bar.
fn solid_bar(ui: &mut egui::Ui, label: &str, used: u64, limit: u64) {
    let text = if limit == 0 {
        format!("{label}: -- / --")
    } else {
        format!("{label}: {:.1} / {:.1} MB", used as f64 / MIB, limit as f64 / MIB)
    };
    let rich =
        egui::RichText::new(text).font(egui::FontId::monospace(10.0)).color(colors::TEXT_SECONDARY);
    ui.label(rich);
    if limit == 0 {
        return;
    }
    let w = ui.available_width().max(80.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 10.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, egui::Rounding::same(0.0), colors::BORDER_SUBTLE);
    if used > 0 {
        let fraction = (used as f64 / limit as f64).clamp(0.0, 1.0) as f32;
        let width = (rect.width() * fraction).max(4.0).min(rect.width());
        let fill = egui::Rect::from_min_size(rect.min, egui::vec2(width, rect.height()));
        ui.painter().rect_filled(fill, egui::Rounding::same(0.0), colors::ACCENT_CYAN);
    }
}

fn render_adapter_card(
    ui: &mut egui::Ui,
    adapters: &[DxgiGpuAdapter],
    snap: &GpuTelemetrySnapshot,
    raw: &[RawProcessCpuRow],
) {
    card_frame().show(ui, |ui| {
        ui.set_min_height(UPPER_H - 18.0);
        crate::ui::primitives::pill_badge(ui, "GPU & VRAM", colors::ACCENT_CYAN);
        let Some(a) = adapters.first() else {
            ui.label(
                egui::RichText::new("No hardware GPU adapter reported")
                    .color(colors::TEXT_SECONDARY),
            );
            return;
        };
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&a.name)
                    .font(egui::FontId::monospace(13.0))
                    .strong()
                    .color(colors::TEXT_PRIMARY),
            );
            let (pill, color) = pill_label(a.integrated);
            egui::Frame::none()
                .fill(color.gamma_multiply(0.16))
                .rounding(egui::Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(pill)
                            .font(egui::FontId::monospace(10.0))
                            .strong()
                            .color(color),
                    );
                });
        });
        ui.label(
            egui::RichText::new(format!(
                "LUID 0x{:016X} | {}",
                a.luid,
                pci_id(a.vendor_id, a.device_id)
            ))
            .font(egui::FontId::monospace(10.0))
            .color(colors::TEXT_SECONDARY),
        );
        ui.add_space(4.0);
        solid_bar(ui, "DEDICATED VRAM", a.dedicated_used_bytes, a.dedicated_limit_bytes);
        solid_bar(ui, "SHARED RAM", a.shared_used_bytes, a.shared_limit_bytes);
        let engine = system_gpu_chart::total_gpu_pct(snap, adapters)
            .map_or_else(|| "GPU ENGINE: --".to_owned(), |g| format!("GPU ENGINE: {g:.1}%"));
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(engine)
                .font(egui::FontId::monospace(10.0))
                .color(colors::ACCENT_CYAN),
        );
        if let Some(top) = snap.processes.values().max_by_key(|p| p.local_bytes) {
            let name =
                raw.iter().find(|p| p.pid == top.pid).map(|p| p.name.as_ref()).unwrap_or("unknown");
            ui.label(
                egui::RichText::new(format!(
                    "TOP VRAM: {name} ({:.1} MB)",
                    top.local_bytes as f64 / MIB
                ))
                .font(egui::FontId::monospace(10.0))
                .color(colors::TEXT_SECONDARY),
            );
        }
    });
}

/// Pure upper-row split: bounded, non-zero widths for both cockpit columns.
fn upper_column_widths(available: f32, gap: f32) -> (f32, f32) {
    let usable = (available - gap).max(240.0);
    let left = (usable * 0.42).clamp(usable * 0.30, usable * 0.55);
    (left.max(120.0), (usable - left).max(160.0))
}

/// Renders the live System tab with GPU cockpit and two read-only diagnostic views.
pub fn render_system_gpu_tab(
    ui: &mut egui::Ui,
    snapshot: &Option<Arc<SystemSnapshot>>,
    cpu_state: &mut CpuTableState,
    cmds: &mut Vec<AppCommand>,
) {
    let mode = render_compact_bar(ui);
    match mode {
        SystemViewMode::GpuVram => {
            let adapters = background_adapters();
            let snap = snapshot.as_ref().and_then(|s| s.gpu.clone()).unwrap_or_default();
            let uptime_secs = snapshot.as_ref().map_or(0, |s| s.uptime_secs);
            let (left, right) =
                upper_column_widths(ui.available_width(), ui.spacing().item_spacing.x);
            // Force top_down: allocate_ui inside horizontal inherits the row layout.
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(left, UPPER_H),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| render_adapter_card(ui, &adapters, &snap, raw_processes(snapshot)),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(right, UPPER_H),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| system_gpu_chart::render_gpu_chart(ui, &adapters, &snap, uptime_secs),
                );
            });
            ui.add_space(6.0);
            system_gpu_table::render_gpu_table(
                ui,
                raw_processes(snapshot),
                &snap,
                &adapters,
                cpu_state,
                cmds,
            );
        }
        SystemViewMode::Services => render_services_view(ui, cpu_state, cmds),
        SystemViewMode::Startup => render_startup_view(ui),
    }
}

fn raw_processes(snapshot: &Option<Arc<SystemSnapshot>>) -> &[RawProcessCpuRow] {
    snapshot.as_ref().and_then(|s| s.cpu.as_ref()).map(|c| c.raw_processes.as_ref()).unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pci_id_is_uppercase_hex() {
        assert_eq!(pci_id(0x10DE, 0x2882), "PCI 10DE:2882");
        assert_eq!(pci_id(0x8086, 0x0007), "PCI 8086:0007");
    }

    #[test]
    fn pill_is_cyan_or_emerald_never_red() {
        assert_eq!(pill_label(false).0, "Discrete GPU");
        assert_eq!(pill_label(false).1, colors::ACCENT_CYAN);
        assert_eq!(pill_label(true).1, OPTIMAL_EMERALD);
    }

    #[test]
    fn upper_columns_are_bounded_and_non_zero() {
        let (left, right) = upper_column_widths(980.0, 8.0);
        assert!(left >= 120.0 && right >= 160.0);
        assert!((left + right - 972.0).abs() < 1.0);
        let (min_left, min_right) = upper_column_widths(0.0, 8.0);
        assert!(min_left > 0.0 && min_right > 0.0);
    }

    #[test]
    fn gpu_tab_headless_render_is_not_blank() {
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0));
        input.screen_rect = Some(rect);
        let mut cpu_state = CpuTableState::default();
        let mut cmds = Vec::new();
        let snapshot: Option<Arc<SystemSnapshot>> = None;
        let output = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_system_gpu_tab(ui, &snapshot, &mut cpu_state, &mut cmds);
            });
        });
        assert!(output.shapes.len() >= 20, "cockpit collapsed to a blank view");
        let text_pos = |needle: &str| {
            output.shapes.iter().find_map(|cs| match &cs.shape {
                egui::epaint::Shape::Text(t) if t.galley.job.text.contains(needle) => Some(t.pos),
                _ => None,
            })
        };
        let chart = text_pos("GPU & VRAM UTILIZATION HISTORY").expect("chart title missing");
        let table = text_pos("PROCESS GPU & VRAM CONSUMERS").expect("table title missing");
        assert!(chart.y < 760.0, "chart pushed off-screen at y={}", chart.y);
        assert!(table.y < 760.0, "consumers table pushed off-screen at y={}", table.y);
    }
}

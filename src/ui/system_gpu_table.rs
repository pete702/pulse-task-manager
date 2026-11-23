//! Full-width sortable per-process GPU and VRAM consumer table.
use std::sync::{Mutex, OnceLock};

use eframe::egui;

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::platform::dxgi_gpu::DxgiGpuAdapter;
use crate::platform::pdh_gpu::{detect_ai_model_tag, is_vram_spill, GpuTelemetrySnapshot};
use crate::theme::{card_frame, colors};
use colors::{ACCENT_CYAN, TEXT_SECONDARY};
const MIB: f64 = 1_048_576.0;
const ACTIVE_LOCAL: u64 = 1024 * 1024;
const ACTIVE_SHARED: u64 = 16 * 1024 * 1024;
const ACTIVE_GPU: f32 = 0.5;
const MAX_ROWS: usize = 24;
#[derive(Clone, Copy, PartialEq, Eq)]
enum GpuSortCol {
    Pid,
    Name,
    Gpu,
    Local,
    Shared,
    Paged,
}
#[derive(Clone, Default)]
struct GpuRow {
    pid: u32,
    name: String,
    gpu: Option<f32>,
    local: u64,
    shared: u64,
    paged: u64,
    discrete: bool,
    flag: String,
}

struct TableUi {
    sort: GpuSortCol,
    asc: bool,
    active_only: bool,
    search: String,
}
impl TableUi {
    fn new() -> Self {
        Self { sort: GpuSortCol::Local, asc: false, active_only: false, search: String::new() }
    }
    fn set_sort(&mut self, col: GpuSortCol) {
        let same = self.sort == col;
        self.asc = if same { !self.asc } else { matches!(col, GpuSortCol::Pid | GpuSortCol::Name) };
        self.sort = col;
    }
}
static TABLE: OnceLock<Mutex<TableUi>> = OnceLock::new();

fn table_ui() -> &'static Mutex<TableUi> {
    TABLE.get_or_init(|| Mutex::new(TableUi::new()))
}

fn mib(bytes: u64) -> String {
    match bytes == 0 {
        true => "--".into(),
        false => format!("{:.1} MB", bytes as f64 / MIB),
    }
}
/// Active = local >= 1 MiB OR shared >= 16 MiB OR GPU >= 0.5%.
fn is_active(r: &GpuRow) -> bool {
    r.local >= ACTIVE_LOCAL || r.shared >= ACTIVE_SHARED || r.gpu.is_some_and(|g| g >= ACTIVE_GPU)
}
type Snap = GpuTelemetrySnapshot;
/// Unknown or iGPU LUIDs never qualify for a VRAM-spill badge.
fn discrete_for_luid(luid: Option<u64>, adapters: &[DxgiGpuAdapter]) -> bool {
    luid.is_some_and(|l| adapters.iter().any(|a| a.luid == l && !a.integrated))
}

fn build_rows(ps: &[RawProcessCpuRow], snap: &Snap, ad: &[DxgiGpuAdapter]) -> Vec<GpuRow> {
    let mut out = Vec::new();
    for p in ps {
        for t in snap.processes.values().filter(|t| t.pid == p.pid) {
            out.push(GpuRow {
                pid: p.pid,
                name: p.name.to_string(),
                gpu: t.gpu_pct,
                local: if t.local_valid { t.local_bytes } else { 0 },
                shared: if t.non_local_valid { t.non_local_bytes } else { 0 },
                paged: t.paged_est_bytes,
                discrete: discrete_for_luid(t.luid, ad),
                flag: detect_ai_model_tag(&p.name, p.command_line.as_deref().unwrap_or(""))
                    .unwrap_or_default(),
            });
        }
    }
    out
}
fn spill_risk(discrete: bool, non_local: u64) -> bool {
    discrete && is_vram_spill(non_local)
}
fn flags_for(r: &GpuRow) -> String {
    let mut out = String::new();
    if spill_risk(r.discrete, r.shared) {
        out.push_str("[ Non-Local >128M / Spill Risk ]");
    }
    if !r.flag.is_empty() {
        out.push_str(if out.is_empty() { "" } else { " " });
        out.push_str(&r.flag);
    }
    out
}
fn sort_rows(rows: &mut [GpuRow], col: GpuSortCol, asc: bool) {
    rows.sort_unstable_by(|a, b| match col {
        GpuSortCol::Pid => a.pid.cmp(&b.pid),
        GpuSortCol::Name => a.name.cmp(&b.name),
        GpuSortCol::Gpu => a.gpu.unwrap_or(-1.0).total_cmp(&b.gpu.unwrap_or(-1.0)),
        GpuSortCol::Local => a.local.cmp(&b.local),
        GpuSortCol::Shared => a.shared.cmp(&b.shared),
        GpuSortCol::Paged => a.paged.cmp(&b.paged),
    });
    if !asc {
        rows.reverse();
    }
}

fn filter_rows(mut rows: Vec<GpuRow>, active_only: bool, search: &str) -> Vec<GpuRow> {
    let q = search.trim().to_ascii_lowercase();
    let hit = |r: &GpuRow| r.name.to_ascii_lowercase().contains(&q) || r.pid.to_string() == q;
    rows.retain(|r| (!active_only || is_active(r)) && (q.is_empty() || hit(r)));
    rows
}
/// One clean top strip: title left, `[ Active Only ] [ All ] Search:` right.
pub fn render_table_controls(ui: &mut egui::Ui) {
    let mut st = table_ui().lock().unwrap_or_else(|e| e.into_inner());
    // Bounds the right_to_left child to one line instead of the full panel height.
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("FILTER / SEARCH").monospace().size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::TextEdit::singleline(&mut st.search).hint_text("Search"));
            ui.label(egui::RichText::new("Search:").monospace().size(10.0));
            let all = ui.selectable_label(!st.active_only, "[ All ]").clicked();
            let active = ui.selectable_label(st.active_only, "[ Active Only ]").clicked();
            st.active_only = active || (st.active_only && !all);
        });
    });
}

/// Final grid cell stretches so striped rows fill the whole panel width.
fn fill_rest(ui: &mut egui::Ui) {
    let w = ui.available_width().max(1.0);
    ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
}

fn gpu_color(gpu: Option<f32>) -> egui::Color32 {
    match gpu.is_some_and(|g| g >= ACTIVE_GPU) {
        true => ACCENT_CYAN,
        false => TEXT_SECONDARY,
    }
}

fn sort_header(ui: &mut egui::Ui, label: &str, col: GpuSortCol, current: GpuSortCol, asc: bool) {
    let active = col == current;
    let arrow = ["", " ^", " v"][usize::from(active) * (2 - usize::from(asc))];
    let text = egui::RichText::new(format!("{label}{arrow}"))
        .monospace()
        .size(10.0)
        .color(if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY });
    if ui.add(egui::Button::new(text).frame(false)).clicked() {
        table_ui().lock().unwrap_or_else(|e| e.into_inner()).set_sort(col);
    }
}

/// Full-width consumers table with clickable sort headers and AI/spill flags.
pub fn render_gpu_table(
    ui: &mut egui::Ui,
    processes: &[RawProcessCpuRow],
    snap: &GpuTelemetrySnapshot,
    adapters: &[DxgiGpuAdapter],
) {
    let st = table_ui().lock().unwrap_or_else(|e| e.into_inner());
    let (sort, asc, active_only, search) = (st.sort, st.asc, st.active_only, st.search.clone());
    let mut rows = filter_rows(build_rows(processes, snap, adapters), active_only, &search);
    sort_rows(&mut rows, sort, asc);
    card_frame().show(ui, |ui| {
        ui.label(egui::RichText::new("PROCESS GPU & VRAM CONSUMERS").monospace().size(12.0));
        let height = ui.available_height().max(200.0);
        let area = egui::ScrollArea::vertical().max_height(height).auto_shrink([false, false]);
        area.show(ui, |ui| {
            let grid = egui::Grid::new("system_gpu_consumers_grid").striped(true).num_columns(8);
            grid.spacing([10.0, 4.0]).show(ui, |ui| {
                sort_header(ui, "PID", GpuSortCol::Pid, sort, asc);
                sort_header(ui, "PROCESS NAME", GpuSortCol::Name, sort, asc);
                sort_header(ui, "GPU %", GpuSortCol::Gpu, sort, asc);
                sort_header(ui, "LOCAL VRAM", GpuSortCol::Local, sort, asc);
                sort_header(ui, "SHARED", GpuSortCol::Shared, sort, asc);
                sort_header(ui, "PAGED (est)", GpuSortCol::Paged, sort, asc);
                ui.label(egui::RichText::new("FLAGS / AI").monospace().size(10.0));
                fill_rest(ui);
                ui.end_row();
                for r in rows.iter().take(MAX_ROWS) {
                    ui.label(egui::RichText::new(r.pid.to_string()).monospace().size(10.0));
                    let name = egui::RichText::new(&r.name).monospace().size(10.0);
                    if r.name.eq_ignore_ascii_case("dwm.exe") {
                        ui.label(name).on_hover_text("dwm.exe aggregates shared surfaces");
                    } else {
                        ui.label(name);
                    }
                    let gc = gpu_color(r.gpu);
                    let gpu_text = r.gpu.map_or("--".to_owned(), |g| format!("{g:.1}"));
                    ui.label(egui::RichText::new(gpu_text).monospace().size(10.0).color(gc));
                    ui.label(egui::RichText::new(mib(r.local)).monospace().size(10.0));
                    ui.label(egui::RichText::new(mib(r.shared)).monospace().size(10.0));
                    ui.label(egui::RichText::new(mib(r.paged)).monospace().size(10.0));
                    let flag = flags_for(r);
                    if flag.is_empty() {
                        ui.label("--");
                    } else {
                        let t = egui::RichText::new(flag).monospace().color(ACCENT_CYAN);
                        ui.label(t);
                    }
                    fill_rest(ui);
                    ui.end_row();
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(pid: u32, gpu: f32, local: u64) -> GpuRow {
        GpuRow { pid, name: format!("p{pid}.exe"), gpu: Some(gpu), local, ..GpuRow::default() }
    }

    #[test]
    fn sorting_and_active_only_filter() {
        let rows = vec![row(1, 0.0, 0), row(2, 3.0, 4 * MIB as u64)];
        assert_eq!(filter_rows(rows.clone(), true, "").len(), 1);
        assert_eq!(filter_rows(rows.clone(), false, "").len(), 2);
        let mut visible = filter_rows(rows, false, "");
        sort_rows(&mut visible, GpuSortCol::Gpu, false);
        assert_eq!(visible[0].pid, 2);
        sort_rows(&mut visible, GpuSortCol::Local, true);
        assert_eq!(visible[0].pid, 1);
    }

    #[test]
    fn igpu_and_unknown_adapters_suppress_spill_badge() {
        let base = GpuRow {
            pid: 9,
            name: "p9.exe".into(),
            gpu: Some(1.0),
            local: 4 * MIB as u64,
            shared: 200 * MIB as u64,
            discrete: true,
            ..GpuRow::default()
        };
        assert!(flags_for(&base).contains("Spill Risk"));
        let igpu = GpuRow { discrete: false, ..base.clone() };
        assert!(!flags_for(&igpu).contains("Spill"));
        let unknown = GpuRow { gpu: None, ..igpu };
        assert_eq!(flags_for(&unknown), "");
    }

    #[test]
    fn dgpu_row_never_inherits_igpu_spill_badge() {
        assert!(spill_risk(true, 150 * MIB as u64));
        assert!(!spill_risk(true, 0));
        assert!(!spill_risk(false, 150 * MIB as u64));
    }

    #[test]
    fn none_gpu_pct_is_not_flattened_to_zero() {
        use crate::platform::pdh_gpu_counters::ProcessGpuTelemetry;
        let proc = RawProcessCpuRow { pid: 5, name: "p5.exe".into(), ..Default::default() };
        let mut snap = GpuTelemetrySnapshot::default();
        snap.processes.insert((5, 0), ProcessGpuTelemetry { pid: 5, ..Default::default() });
        assert_eq!(build_rows(std::slice::from_ref(&proc), &snap, &[])[0].gpu, None);
        snap.processes.get_mut(&(5, 0)).unwrap().gpu_pct = Some(0.0);
        assert_eq!(build_rows(&[proc], &snap, &[])[0].gpu, Some(0.0));
    }
}

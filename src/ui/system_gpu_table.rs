//! Full-width sortable per-process GPU and VRAM consumer table.
use std::sync::{Mutex, OnceLock};

use eframe::egui;

use crate::app_cmd::AppCommand;
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::cpu::table_state::CpuTableState;
use crate::platform::dxgi_gpu::DxgiGpuAdapter;
use crate::platform::pdh_gpu::{detect_ai_model_tag, is_vram_spill, GpuTelemetrySnapshot};
use crate::theme::{card_frame, colors};
use crate::ui::primitives::pill_button_colored;
use colors::ACCENT_CYAN;
const MIB: f64 = 1_048_576.0;
const ACTIVE_LOCAL: u64 = 1024 * 1024;
const ACTIVE_SHARED: u64 = 16 * 1024 * 1024;
const ACTIVE_GPU: f32 = 0.5;
type GpuSortCol = u8;
const COL_PID: u8 = 0;
const COL_NAME: u8 = 1;
const COL_GPU: u8 = 2;
const COL_LOCAL: u8 = 3;
const COL_SHARED: u8 = 4;
const COL_PAGED: u8 = 5;
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
        Self { sort: COL_LOCAL, asc: false, active_only: false, search: String::new() }
    }
    fn set_sort(&mut self, col: GpuSortCol) {
        self.asc = if self.sort == col { !self.asc } else { matches!(col, COL_PID | COL_NAME) };
        self.sort = col;
    }
}
static TABLE: OnceLock<Mutex<TableUi>> = OnceLock::new();
fn table_ui() -> &'static Mutex<TableUi> {
    TABLE.get_or_init(|| Mutex::new(TableUi::new()))
}
fn mib(bytes: u64) -> String {
    if bytes == 0 {
        "--".into()
    } else {
        format!("{:.1} MB", bytes as f64 / MIB)
    }
}
fn is_active(r: &GpuRow) -> bool {
    r.local >= ACTIVE_LOCAL || r.shared >= ACTIVE_SHARED || r.gpu.is_some_and(|g| g >= ACTIVE_GPU)
}
fn discrete_for_luid(luid: Option<u64>, adapters: &[DxgiGpuAdapter]) -> bool {
    luid.is_some_and(|l| adapters.iter().any(|a| a.luid == l && !a.integrated))
}
fn build_rows(
    ps: &[RawProcessCpuRow],
    snap: &GpuTelemetrySnapshot,
    ad: &[DxgiGpuAdapter],
) -> Vec<GpuRow> {
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
fn flags_for(r: &GpuRow) -> String {
    let mut out = String::new();
    if r.discrete && is_vram_spill(r.shared) {
        out.push_str("Non-Local >128M / Spill Risk");
    }
    if !r.flag.is_empty() {
        if !out.is_empty() {
            out.push_str(" | ");
        }
        out.push_str(r.flag.trim_start_matches("[ ").trim_end_matches(" ]"));
    }
    out
}
fn sort_rows(rows: &mut [GpuRow], col: GpuSortCol, asc: bool) {
    rows.sort_unstable_by(|a, b| match col {
        COL_PID => a.pid.cmp(&b.pid),
        COL_NAME => a.name.cmp(&b.name),
        COL_GPU => a.gpu.unwrap_or(-1.0).total_cmp(&b.gpu.unwrap_or(-1.0)),
        COL_LOCAL => a.local.cmp(&b.local),
        COL_SHARED => a.shared.cmp(&b.shared),
        COL_PAGED => a.paged.cmp(&b.paged),
        _ => a.pid.cmp(&b.pid),
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
pub fn render_gpu_filter(ui: &mut egui::Ui) {
    let mut st = table_ui().lock().unwrap_or_else(|e| e.into_inner());
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add(egui::TextEdit::singleline(&mut st.search).hint_text("Search").desired_width(120.0));
        let all_col = if !st.active_only { ACCENT_CYAN } else { colors::TEXT_SECONDARY };
        let active_col = if st.active_only { ACCENT_CYAN } else { colors::TEXT_SECONDARY };
        let all = pill_button_colored(ui, "All", all_col).clicked();
        let active = pill_button_colored(ui, "Active Only", active_col).clicked();
        st.active_only = active || (st.active_only && !all);
    });
}
fn sort_header(ui: &mut egui::Ui, label: &str, col: GpuSortCol, current: GpuSortCol, asc: bool) {
    let active = col == current;
    let arrow = ["", " ^", " v"][usize::from(active) * (2 - usize::from(asc))];
    let text = egui::RichText::new(format!("{label}{arrow}"))
        .font(egui::FontId::proportional(10.5))
        .color(if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY });
    if ui.add(egui::Button::new(text).frame(false)).clicked() {
        table_ui().lock().unwrap_or_else(|e| e.into_inner()).set_sort(col);
    }
}
fn pid_pill(ui: &mut egui::Ui, pid: u32) -> bool {
    let text =
        egui::RichText::new(pid.to_string()).font(egui::FontId::monospace(10.0)).color(ACCENT_CYAN);
    ui.add(egui::Button::new(text).rounding(egui::Rounding::same(colors::PILL_RADIUS))).clicked()
}
pub fn render_gpu_table(
    ui: &mut egui::Ui,
    processes: &[RawProcessCpuRow],
    snap: &GpuTelemetrySnapshot,
    adapters: &[DxgiGpuAdapter],
    cpu_state: &mut CpuTableState,
    cmds: &mut Vec<AppCommand>,
) {
    let st = table_ui().lock().unwrap_or_else(|e| e.into_inner());
    let (sort, asc, active_only, search) = (st.sort, st.asc, st.active_only, st.search.clone());
    let mut rows = filter_rows(build_rows(processes, snap, adapters), active_only, &search);
    sort_rows(&mut rows, sort, asc);
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new("PROCESS GPU & VRAM CONSUMERS")
                .font(egui::FontId::proportional(12.0)),
        );
        let height = ui.available_height().max(200.0);
        egui::ScrollArea::vertical().max_height(height).auto_shrink([false, false]).show(
            ui,
            |ui| {
                let grid =
                    egui::Grid::new("system_gpu_consumers_grid").striped(true).num_columns(8);
                grid.spacing([10.0, 4.0]).show(ui, |ui| {
                    sort_header(ui, "PID", COL_PID, sort, asc);
                    sort_header(ui, "PROCESS NAME", COL_NAME, sort, asc);
                    sort_header(ui, "GPU %", COL_GPU, sort, asc);
                    sort_header(ui, "LOCAL VRAM", COL_LOCAL, sort, asc);
                    sort_header(ui, "SHARED", COL_SHARED, sort, asc);
                    sort_header(ui, "PAGED (est)", COL_PAGED, sort, asc);
                    ui.label(
                        egui::RichText::new("FLAGS / AI").font(egui::FontId::proportional(10.5)),
                    );
                    ui.allocate_exact_size(
                        egui::vec2(ui.available_width().max(1.0), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.end_row();
                    for r in &rows {
                        let pid_click = pid_pill(ui, r.pid);
                        let name = ui.add(
                            egui::Button::new(
                                egui::RichText::new(&r.name).font(egui::FontId::proportional(10.5)),
                            )
                            .frame(false),
                        );
                        if pid_click || name.clicked() {
                            cpu_state.search_query = r.pid.to_string();
                            cpu_state.selected_pid = Some(r.pid);
                            cpu_state.mark_dirty();
                            cmds.push(AppCommand::SetTab(ActiveTab::Processes));
                            cmds.push(AppCommand::RequestProcessScan);
                        }
                        let gc = if r.gpu.is_some_and(|g| g >= ACTIVE_GPU) {
                            ACCENT_CYAN
                        } else {
                            colors::TEXT_SECONDARY
                        };
                        let gpu_text = r.gpu.map_or("--".to_owned(), |g| format!("{g:.1}"));
                        ui.label(
                            egui::RichText::new(gpu_text)
                                .font(egui::FontId::monospace(10.0))
                                .color(gc),
                        );
                        ui.label(
                            egui::RichText::new(mib(r.local)).font(egui::FontId::monospace(10.0)),
                        );
                        ui.label(
                            egui::RichText::new(mib(r.shared)).font(egui::FontId::monospace(10.0)),
                        );
                        ui.label(
                            egui::RichText::new(mib(r.paged)).font(egui::FontId::monospace(10.0)),
                        );
                        let flag = flags_for(r);
                        if flag.is_empty() {
                            ui.label("--");
                        } else {
                            ui.label(
                                egui::RichText::new(flag)
                                    .font(egui::FontId::proportional(10.5))
                                    .color(ACCENT_CYAN),
                            );
                        }
                        ui.end_row();
                    }
                });
            },
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(pid: u32, gpu: f32, local: u64) -> GpuRow {
        GpuRow { pid, name: format!("p{pid}.exe"), gpu: Some(gpu), local, ..GpuRow::default() }
    }
    #[test]
    fn gpu_table_rules() {
        use crate::platform::pdh_gpu_counters::ProcessGpuTelemetry;
        let rows = vec![row(1, 0.0, 0), row(2, 3.0, 4 * MIB as u64)];
        assert_eq!(filter_rows(rows.clone(), true, "").len(), 1);
        assert_eq!(filter_rows(rows.clone(), false, "").len(), 2);
        let mut v = filter_rows(rows, false, "");
        sort_rows(&mut v, COL_GPU, false);
        assert_eq!(v[0].pid, 2);
        sort_rows(&mut v, COL_LOCAL, true);
        assert_eq!(v[0].pid, 1);
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
        assert!(!flags_for(&GpuRow { discrete: false, ..base.clone() }).contains("Spill"));
        assert_eq!(flags_for(&GpuRow { gpu: None, discrete: false, ..base }), "");
        assert!(is_vram_spill(150 * MIB as u64) && !is_vram_spill(MIB as u64));
        let proc = RawProcessCpuRow { pid: 5, name: "p5.exe".into(), ..Default::default() };
        let mut snap = GpuTelemetrySnapshot::default();
        snap.processes.insert((5, 0), ProcessGpuTelemetry { pid: 5, ..Default::default() });
        assert_eq!(build_rows(std::slice::from_ref(&proc), &snap, &[])[0].gpu, None);
        snap.processes.get_mut(&(5, 0)).unwrap().gpu_pct = Some(0.0);
        assert_eq!(build_rows(&[proc], &snap, &[])[0].gpu, Some(0.0));
    }
}

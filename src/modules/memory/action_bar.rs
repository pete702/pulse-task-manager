//! Non-blocking memory action bar: working-set trim, open location, and end task.
use crate::modules::memory::actions::{self, ReclaimReport};
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::process_model::ProcessMemoryItem;
use crate::modules::memory::safe_handle::{is_app_elevated, verify_pid_safety_gate};
use crate::modules::memory::table_view::MemoryTableState;
use crate::theme::{colors, TEXT_MUTED};
use eframe::egui;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub const STATUS_TTL: Duration = Duration::from_secs(8);
pub const CONFIRM_TTL: Duration = Duration::from_secs(4);
fn elevated() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(is_app_elevated)
}
pub fn format_trim_status(r: &ReclaimReport) -> String {
    let s = r.skipped_protected + r.skipped_access_denied;
    format!(
        "Working-set trim: {:.1} MB across {} procs ({} skipped)",
        r.reclaimed_bytes as f64 / 1_048_576.0,
        r.trimmed_count,
        s
    )
}
#[derive(Debug, Default)]
pub struct MemoryActionBarState {
    pending_rx: Option<Receiver<String>>,
    last_status: Option<(Instant, String)>,
    confirm_kill_group: Option<(String, Instant)>,
}
impl MemoryActionBarState {
    pub fn set_status(&mut self, now: Instant, text: String) {
        self.last_status = Some((now, text));
    }
    pub fn status_text(&self, now: Instant) -> Option<&str> {
        self.last_status
            .as_ref()
            .and_then(|(at, t)| (now.duration_since(*at) < STATUS_TTL).then_some(t.as_str()))
    }
    pub fn begin_confirm_kill(&mut self, g: &str, now: Instant) {
        self.confirm_kill_group = Some((g.to_owned(), now));
    }
    pub fn cancel_confirm(&mut self) {
        self.confirm_kill_group = None;
    }
    pub fn confirm_kill_active(&self, g: &str, now: Instant) -> bool {
        self.confirm_kill_group
            .as_ref()
            .is_some_and(|(gr, at)| gr == g && now.duration_since(*at) < CONFIRM_TTL)
    }
    fn poll(&mut self, now: Instant) {
        if let Some(rx) = self.pending_rx.take() {
            match rx.try_recv() {
                Ok(t) => self.last_status = Some((now, t)),
                Err(TryRecvError::Empty) => self.pending_rx = Some(rx),
                Err(TryRecvError::Disconnected) => {
                    self.last_status = Some((now, "Action thread stopped".to_owned()))
                }
            }
        }
        if self.last_status.as_ref().is_some_and(|(at, _)| now.duration_since(*at) >= STATUS_TTL) {
            self.last_status = None;
        }
        if self
            .confirm_kill_group
            .as_ref()
            .is_some_and(|(_, at)| now.duration_since(*at) >= CONFIRM_TTL)
        {
            self.confirm_kill_group = None;
        }
    }
    fn spawn(&mut self, job: impl FnOnce() -> String + Send + 'static) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(job());
        });
        self.pending_rx = Some(rx);
    }
}
fn group_pids(m: &MemorySnapshot, g: &str) -> Vec<u32> {
    m.processes.iter().filter(|p| p.1 == g).map(|p| p.0).collect()
}
fn group_targets(rows: &[ProcessMemoryItem], g: &str) -> Vec<ProcessMemoryItem> {
    rows.iter().filter(|r| r.name == g).cloned().collect()
}
pub fn render_action_bar(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    rows: &[ProcessMemoryItem],
    ts: &mut MemoryTableState,
    st: &mut MemoryActionBarState,
) {
    let now = Instant::now();
    st.poll(now);
    let sel = ts.selected_group.clone();
    crate::theme::card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            let (b, c) = if elevated() {
                ("[ ADMIN MODE ]", colors::ACCENT_CYAN)
            } else {
                ("[ USER MODE ]", TEXT_MUTED)
            };
            ui.label(egui::RichText::new(b).monospace().size(10.0).color(c))
                .on_hover_text("Elevation decides which processes may be trimmed or ended");
            ui.separator();
            if ui.add_enabled(!rows.is_empty(), egui::Button::new("⚡ Trim Working Sets")).clicked()
            {
                let items = rows.to_vec();
                st.spawn(move || {
                    let r = actions::purge_system_working_sets(&items);
                    let _ = crate::core::win32::self_trim_working_set();
                    format_trim_status(&r)
                });
            }
            if let Some(g) = sel {
                ui.separator();
                let t = group_targets(rows, &g);
                let p = group_pids(mem, &g);
                ui.label(egui::RichText::new(&g).monospace().size(11.0).color(colors::ACCENT_CYAN));
                if ui.add_enabled(!t.is_empty(), egui::Button::new("Trim Selected")).clicked() {
                    let items = t.clone();
                    st.spawn(move || format_trim_status(&actions::trim_process_group(&items)));
                }
                let path = t.iter().find_map(|r| r.exe_path.clone().filter(|p| !p.is_empty()));
                if ui.add_enabled(path.is_some(), egui::Button::new("Open Location")).clicked() {
                    if let Some(p) = path {
                        let msg =
                            match actions::open_process_file_location(std::path::Path::new(&p)) {
                                Ok(()) => format!("Opened location for {g}"),
                                Err(e) => format!("Open location failed: {e}"),
                            };
                        st.set_status(Instant::now(), msg);
                    }
                }
                let a = st.confirm_kill_active(&g, now);
                let pr =
                    p.is_empty() || p.iter().any(|pid| verify_pid_safety_gate(*pid, &g).is_err());
                let lbl = if a { "Confirm Kill?" } else { "End Task" };
                if ui.add_enabled(!pr, egui::Button::new(lbl)).clicked() {
                    if a {
                        let items = t;
                        st.cancel_confirm();
                        st.spawn(move || match actions::terminate_process_group(&p, &items) {
                            Ok(n) => format!("End task: {n}/{} processes terminated", p.len()),
                            Err(e) => format!("End task blocked: {e}"),
                        });
                    } else {
                        st.begin_confirm_kill(&g, now);
                    }
                }
            }
        });
        if let Some(t) = st.status_text(Instant::now()) {
            ui.label(egui::RichText::new(t).monospace().size(10.0).color(colors::TEXT_SECONDARY));
        }
    });
}

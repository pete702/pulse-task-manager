//! PULSE rendering helpers — split from app.rs for modularity and line limits.

use std::sync::Arc;

use eframe::egui;

use crate::app_tabs::ActiveTab;
use crate::core::SystemSnapshot;
use crate::i18n::Key;
use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::page::{self, MemorySubView};
use crate::modules::memory::table_state::MemoryTableState;
use crate::ui::{chrome, sidebar};

/// Renders the central panel content based on the active tab.
#[allow(clippy::too_many_arguments)]
pub fn render_central_panel(
    ui: &mut egui::Ui,
    active_tab: ActiveTab,
    memory_sub_view: &mut MemorySubView,
    table_state: &mut MemoryTableState,
    action_state: &mut MemoryActionBarState,
    memory_history: &mut MemoryHistory,
    snapshot: &Option<Arc<SystemSnapshot>>,
    uptime_secs: u64,
    telemetry_paused: &mut bool,
) {
    ui.add_space(16.0);
    match active_tab {
        ActiveTab::Memory => {
            if let Some(ref snap) = snapshot {
                if let Some(ref mem) = snap.memory {
                    page::render_memory_page(
                        ui,
                        mem,
                        memory_history,
                        memory_sub_view,
                        table_state,
                        action_state,
                        uptime_secs,
                        telemetry_paused,
                    );
                } else {
                    render_awaiting(ui);
                }
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::Cpu => chrome::render_placeholder_tab(ui, Key::TabCpu.t(), "v0.2.0"),
        ActiveTab::Storage => chrome::render_placeholder_tab(ui, Key::TabStorage.t(), "v0.3.0"),
        ActiveTab::Network => chrome::render_placeholder_tab(ui, Key::TabNetwork.t(), "v0.4.0"),
        ActiveTab::System => chrome::render_placeholder_tab(ui, Key::TabSystem.t(), "v0.5.0"),
    }
}

/// Renders the awaiting memory model message.
fn render_awaiting(ui: &mut egui::Ui) {
    let text = egui::RichText::new(Key::AwaitingMemoryModel.t()).size(14.0);
    ui.label(text.color(crate::theme::colors::TEXT_SECONDARY));
}

/// Renders the top header panel.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
) {
    chrome::render_top_panel(ui, snapshot, last_ui_ms);
}

/// Renders the bottom status bar.
pub fn render_status_bar(
    ui: &mut egui::Ui,
    active_processes: u32,
    snapshot: &Option<Arc<SystemSnapshot>>,
) {
    chrome::render_status_bar(ui, active_processes, snapshot);
}

/// Renders the left sidebar.
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    snapshot: &Option<Arc<SystemSnapshot>>,
) {
    sidebar::render_sidebar(ui, active_tab, snapshot);
}

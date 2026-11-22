//! Centered command palette (Ctrl+K): fuzzy command and live-process launcher.
use std::sync::{Mutex, OnceLock};

use eframe::egui;

use crate::app_cmd::AppCommand;
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::cpu::table_state::CpuTableState;
use crate::theme::colors;

#[path = "command_palette_logic.rs"]
mod logic;
use logic::{
    build_items, clamp_palette_selection, matches_palette_query, move_palette_selection,
    PaletteAction,
};

const INPUT_ID: &str = "command_palette_search";

#[derive(Default)]
struct PaletteState {
    open: bool,
    query: String,
    selected_idx: usize,
    just_opened: bool,
}

static STATE: OnceLock<Mutex<PaletteState>> = OnceLock::new();

fn state() -> &'static Mutex<PaletteState> {
    STATE.get_or_init(|| Mutex::new(PaletteState::default()))
}

fn close_state(state: &mut PaletteState) {
    state.open = false;
    state.just_opened = false;
    state.selected_idx = 0;
    state.query.clear();
}

/// Toggles visibility, resetting transient state on each open.
pub fn toggle() {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    if guard.open {
        close_state(&mut guard);
    } else {
        guard.open = true;
        guard.just_opened = true;
        guard.selected_idx = 0;
        guard.query.clear();
    }
}

pub fn is_open() -> bool {
    state().lock().unwrap_or_else(|e| e.into_inner()).open
}

fn execute(
    action: PaletteAction,
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    cpu_state: &mut CpuTableState,
    cmds: &mut Vec<AppCommand>,
) {
    match action {
        PaletteAction::Tab(tab) => *active_tab = tab,
        PaletteAction::ToggleMiniHud => {
            crate::platform::hotkey::set_mini_hud_active(
                !crate::platform::hotkey::is_mini_hud_active(),
            );
        }
        PaletteAction::TrimRam => cmds.push(AppCommand::RequestMemoryTrim),
        PaletteAction::ExportReceipt => {
            let _ = crate::core::blackbox::export_lag_receipt();
        }
        PaletteAction::PickWindow => {
            *active_tab = ActiveTab::Cpu;
            crate::platform::pick_window::arm_pick_window();
        }
        PaletteAction::HideToTray => cmds.push(AppCommand::HideToTray),
        PaletteAction::Exit => {
            crate::app_tray::set_allow_real_close(true);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        PaletteAction::SelectProcess(pid) => {
            *active_tab = ActiveTab::Cpu;
            cpu_state.search_query.clear();
            cpu_state.selected_pid = Some(pid);
            cpu_state.selected_start_time = None;
            cpu_state.selected_name = None;
            cpu_state.mark_dirty();
        }
    }
    ctx.request_repaint();
}

/// Renders the palette modal and handles its keyboard navigation. Call every frame.
pub fn render_command_palette(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    cpu_state: &mut CpuTableState,
    cmds: &mut Vec<AppCommand>,
    processes: &[RawProcessCpuRow],
) {
    let holder = state();
    let mut guard = holder.lock().unwrap_or_else(|e| e.into_inner());
    if !guard.open {
        return;
    }

    let items = build_items(processes, &guard.query);
    let visible: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| matches_palette_query(&guard.query, &item.label))
        .map(|(idx, _)| idx)
        .collect();
    guard.selected_idx = clamp_palette_selection(guard.selected_idx, visible.len());

    let (up, down, enter, escape) = ctx.input(|i| {
        (
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowDown),
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Escape),
        )
    });
    if down {
        guard.selected_idx = move_palette_selection(guard.selected_idx, visible.len(), 1);
    }
    if up {
        guard.selected_idx = move_palette_selection(guard.selected_idx, visible.len(), -1);
    }
    if escape {
        close_state(&mut guard);
        return;
    }
    if enter {
        if let Some(&idx) = visible.get(guard.selected_idx) {
            let action = items[idx].action;
            close_state(&mut guard);
            execute(action, ctx, active_tab, cpu_state, cmds);
        }
        return;
    }

    if guard.just_opened {
        ctx.memory_mut(|m| m.request_focus(egui::Id::new(INPUT_ID)));
        guard.just_opened = false;
    }

    let mut query = guard.query.clone();
    let selected = guard.selected_idx;
    let mut pending: Option<PaletteAction> = None;
    let mut selected_response: Option<egui::Response> = None;
    egui::Window::new("COMMAND PALETTE (Ctrl+K)")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(460.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(">").monospace().size(13.0).color(colors::ACCENT_CYAN),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut query)
                        .id(egui::Id::new(INPUT_ID))
                        .hint_text("Search commands and processes...")
                        .desired_width(410.0),
                );
            });
            ui.separator();
            if visible.is_empty() {
                ui.label(
                    egui::RichText::new("No matches").size(11.0).color(colors::TEXT_SECONDARY),
                );
            } else {
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (row, &idx) in visible.iter().enumerate() {
                        let is_selected = row == selected;
                        let text = if is_selected {
                            egui::RichText::new(&items[idx].label)
                                .monospace()
                                .size(12.0)
                                .strong()
                                .color(colors::ACCENT_CYAN)
                        } else {
                            egui::RichText::new(&items[idx].label)
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_PRIMARY)
                        };
                        let response = ui.selectable_label(is_selected, text);
                        if is_selected {
                            selected_response = Some(response.clone());
                        }
                        if response.clicked() {
                            pending = Some(items[idx].action);
                        }
                    }
                });
            }
        });
    if up || down {
        if let Some(response) = selected_response {
            response.scroll_to_me(None);
        }
    }
    guard.query = query;
    if let Some(action) = pending {
        close_state(&mut guard);
        execute(action, ctx, active_tab, cpu_state, cmds);
    }
}

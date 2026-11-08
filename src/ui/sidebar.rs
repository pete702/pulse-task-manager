//! Rich Telemetry Sidebar for PULSE -- Obsidian HUD Navigation Cards.
use crate::core::SystemSnapshot;
use crate::theme::colors;
use crate::ui::sidebar_badges::draw_tab_badge;
use eframe::egui;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
static STORAGE_ACTIVE: AtomicBool = AtomicBool::new(false);
static NETWORK_ACTIVE: AtomicBool = AtomicBool::new(false);
fn next_tab(tab: crate::app_tabs::ActiveTab) -> crate::app_tabs::ActiveTab {
    use crate::app_tabs::ActiveTab as T;
    match tab {
        T::Processes => T::Memory,
        T::Memory => T::Cpu,
        T::Cpu => T::Storage,
        T::Storage => T::Network,
        T::Network => T::System,
        T::System => T::Processes,
    }
}
fn prev_tab(tab: crate::app_tabs::ActiveTab) -> crate::app_tabs::ActiveTab {
    use crate::app_tabs::ActiveTab as T;
    match tab {
        T::Processes => T::System,
        T::Memory => T::Processes,
        T::Cpu => T::Memory,
        T::Storage => T::Cpu,
        T::Network => T::Storage,
        T::System => T::Network,
    }
}
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut crate::app_tabs::ActiveTab,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        let (up, down, confirm) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space),
            )
        });
        let tabs = std::iter::once(crate::app_tabs::ActiveTab::Processes)
            .chain(crate::app_tabs::ActiveTab::all().iter().copied());
        for tab in tabs {
            let is_active = *active_tab == tab;
            let title = tab.label();
            let (bg, border, accent, text_col) = if is_active {
                (
                    egui::Color32::from_rgb(0x16, 0x1F, 0x2E),
                    colors::ACCENT_CYAN,
                    colors::ACCENT_CYAN,
                    colors::TEXT_PRIMARY,
                )
            } else {
                (
                    egui::Color32::from_rgb(0x12, 0x14, 0x1C),
                    colors::BORDER,
                    egui::Color32::TRANSPARENT,
                    colors::TEXT_SECONDARY,
                )
            };
            let available_width = ui.available_width().max(0.0);
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(available_width, 42.0), egui::Sense::click());
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), bg);
            ui.painter().rect_stroke(
                rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0_f32, border),
            );
            if is_active {
                let ar = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top()),
                    egui::pos2(rect.left() + 3.0, rect.bottom()),
                );
                ui.painter().rect_filled(ar, egui::Rounding::same(6.0), accent);
                crate::ui::primitives::draw_cyber_corner_brackets(
                    ui,
                    rect,
                    egui::Color32::from_rgba_unmultiplied(0, 229, 255, 130),
                );
            }
            ui.painter().text(
                egui::pos2(rect.left() + 14.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::monospace(13.0),
                text_col,
            );
            if rect.width() >= 140.0 {
                if let Some(ref snap) = latest_snapshot {
                    draw_tab_badge(ui, rect, tab, snap, &STORAGE_ACTIVE, &NETWORK_ACTIVE);
                }
            }
            if response.clicked() {
                *active_tab = tab;
            }
            if response.has_focus() {
                if confirm {
                    *active_tab = tab;
                } else if down {
                    *active_tab = next_tab(tab);
                } else if up {
                    *active_tab = prev_tab(tab);
                }
            }
            ui.add_space(6.0);
        }
    });
}

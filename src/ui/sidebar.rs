//! Rich Telemetry Sidebar for PULSE — Obsidian HUD Navigation Cards.

use crate::core::SystemSnapshot;
use crate::theme::colors;
use eframe::egui;
use std::sync::Arc;

/// Renders the left sidebar with rich navigation mini-cards.
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut crate::app_tabs::ActiveTab,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        let tabs = crate::app_tabs::ActiveTab::all();
        for tab in tabs {
            let is_active = *active_tab == *tab;
            let title = tab.label();

            // Determine card colors based on active state
            let (bg_color, border_color, accent_color, text_color) = if is_active {
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
                    egui::Color32::from_rgb(0x94, 0xA3, 0xB8), // #94A3B8 - readable inactive
                )
            };

            let card_height = 42.0;
            let available_width = ui.available_width();

            // Render as custom card using interact() for click handling
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(available_width, card_height),
                egui::Sense::click(),
            );

            // Draw card background & border
            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), bg_color);
            ui.painter().rect_stroke(
                rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0_f32, border_color),
            );

            // Draw left accent bar for active tab
            if is_active {
                let accent_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top()),
                    egui::pos2(rect.left() + 3.0, rect.bottom()),
                );
                ui.painter().rect_filled(accent_rect, egui::Rounding::same(6.0), accent_color);
            }

            // Paint tab title at LEFT_CENTER
            let title_pos = egui::pos2(rect.left() + 14.0, rect.center().y);
            ui.painter().text(
                title_pos,
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::monospace(13.0),
                text_color,
            );

            // Paint live RAM % badge on Memory tab at RIGHT_CENTER
            if *tab == crate::app_tabs::ActiveTab::Memory {
                if let Some(ref snapshot) = latest_snapshot {
                    if let Some(ref mem) = snapshot.memory {
                        let ram_pct = mem.ram_usage_percent();
                        let pct_color = crate::theme::load_color(ram_pct);
                        let pct_text = format!("{:.0}%", ram_pct);
                        let pct_pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                        ui.painter().text(
                            pct_pos,
                            egui::Align2::RIGHT_CENTER,
                            pct_text,
                            egui::FontId::monospace(13.0),
                            pct_color,
                        );
                    }
                }
            }

            if response.clicked() {
                *active_tab = *tab;
            }

            ui.add_space(6.0);
        }
    });
}

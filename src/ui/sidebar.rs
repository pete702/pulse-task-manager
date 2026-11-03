//! Rich Telemetry Sidebar for PULSE — Obsidian HUD Mini-Cards.

use crate::core::SystemSnapshot;
use crate::theme::colors;
use eframe::egui;

/// Renders the left sidebar with rich telemetry mini-cards.
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut crate::app_tabs::ActiveTab,
    latest_snapshot: &Option<SystemSnapshot>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        let tabs = crate::app_tabs::ActiveTab::all();
        for tab in tabs {
            let is_active = *active_tab == *tab;
            let btn_text = tab.label();

            // Build display text for Memory tab with live RAM %
            let display_text = if *tab == crate::app_tabs::ActiveTab::Memory {
                if let Some(ref snapshot) = latest_snapshot {
                    if let Some(ref mem) = snapshot.memory {
                        format!("{} [{:.0}%]", btn_text, mem.ram_usage_percent())
                    } else {
                        btn_text.to_string()
                    }
                } else {
                    btn_text.to_string()
                }
            } else {
                btn_text.to_string()
            };

            // Determine card colors based on active state
            let (bg_color, border_color, text_color) = if is_active {
                (
                    egui::Color32::from_rgb(0x16, 0x1F, 0x2E),
                    colors::ACCENT_CYAN,
                    colors::TEXT_PRIMARY,
                )
            } else {
                (
                    egui::Color32::from_rgb(0x12, 0x14, 0x1C),
                    colors::BORDER,
                    colors::TEXT_SECONDARY,
                )
            };

            let card_width = 186.0;
            let card_height = 38.0;

            // Render as custom card using interact() for click handling (no ugly button fill)
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::click());

            // Draw card background & border
            ui.painter()
                .rect_filled(rect, egui::Rounding::same(6.0), bg_color);
            ui.painter().rect_stroke(
                rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0_f32, border_color),
            );

            // Card content
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);

                    // Memory tab gets special multi-row treatment
                    if *tab == crate::app_tabs::ActiveTab::Memory {
                        if let Some(ref snapshot) = latest_snapshot {
                            if let Some(ref mem) = snapshot.memory {
                                // Row 1: Title + live RAM %
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(btn_text)
                                            .monospace()
                                            .size(13.0)
                                            .color(text_color),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let pct = mem.ram_usage_percent();
                                            let color = crate::theme::load_color(pct);
                                            ui.label(
                                                egui::RichText::new(format!("{:.0}%", pct))
                                                    .monospace()
                                                    .size(13.0)
                                                    .color(color),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(2.0);

                                // Row 2: Mini progress bar
                                let pct = mem.ram_usage_percent() / 100.0;
                                let bar_color = crate::theme::load_color(mem.ram_usage_percent());
                                ui.add(
                                    egui::ProgressBar::new(pct)
                                        .fill(bar_color)
                                        .animate(false)
                                        .rounding(egui::Rounding::same(2.0))
                                        .desired_width(160.0)
                                        .show_percentage(),
                                );
                                ui.add_space(1.0);

                                // Row 3: Subtitle
                                let used_gib = mem.ram_used_gib();
                                let total_gib = mem.ram_total_gib();
                                let commit_pct = if mem.commit_limit_bytes > 0 {
                                    (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64
                                        * 100.0) as f32
                                } else {
                                    0.0
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:.2} / {:.2} GiB • Commit {:.0}%",
                                        used_gib, total_gib, commit_pct
                                    ))
                                    .monospace()
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY),
                                );
                            } else {
                                // Fallback if no memory data
                                ui.label(
                                    egui::RichText::new(display_text)
                                        .monospace()
                                        .size(13.0)
                                        .color(text_color),
                                );
                            }
                        } else {
                            // Other tabs: single row with subtitle
                            ui.label(
                                egui::RichText::new(display_text)
                                    .monospace()
                                    .size(13.0)
                                    .color(text_color),
                            );
                            ui.add_space(2.0);
                            let subtitle = match tab {
                                crate::app_tabs::ActiveTab::Cpu => "Telemetry: Phase 2",
                                crate::app_tabs::ActiveTab::Storage => "Telemetry: Phase 3",
                                crate::app_tabs::ActiveTab::Network => "Telemetry: Phase 3",
                                crate::app_tabs::ActiveTab::System => "Telemetry: Phase 3",
                                _ => "",
                            };
                            if !subtitle.is_empty() {
                                ui.label(
                                    egui::RichText::new(subtitle)
                                        .monospace()
                                        .size(10.0)
                                        .color(colors::TEXT_SECONDARY),
                                );
                            }
                        }
                    }
                });
            });

            if response.clicked() {
                *active_tab = *tab;
            }
        }
    });
}

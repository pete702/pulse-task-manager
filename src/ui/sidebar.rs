//! Rich Telemetry Sidebar for PULSE - Obsidian HUD Navigation Cards.

use std::sync::Arc;

use crate::core::SystemSnapshot;
use crate::theme::colors;
use crate::ui::sidebar_badges::{
    badge_active, cached_count, cached_cpu_pct, cached_network_badge, cached_pct,
    cached_storage_badge,
};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};

static STORAGE_ACTIVE: AtomicBool = AtomicBool::new(false);
static NETWORK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Stable accent color with hysteresis above 0.15 / below 0.05.
fn hysteresis_active(slot: &AtomicBool, total: f32) -> bool {
    let mut cur = slot.load(Ordering::Relaxed);
    let next = badge_active(total, &mut cur);
    slot.store(next, Ordering::Relaxed);
    next
}

/// Next tab in sidebar order (wraps).
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

/// Previous tab in sidebar order (wraps).
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

/// Renders the left sidebar with rich navigation mini-cards.
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut crate::app_tabs::ActiveTab,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        let tabs = std::iter::once(crate::app_tabs::ActiveTab::Processes)
            .chain(crate::app_tabs::ActiveTab::all().iter().copied());
        for tab in tabs {
            let is_active = *active_tab == tab;
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
                    colors::TEXT_SECONDARY,
                )
            };

            let card_height = 42.0;
            let available_width = ui.available_width();

            // Render as custom card using interact() for click handling
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(available_width, card_height),
                egui::Sense::click(),
            );
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

            // Draw card background & border
            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), bg_color);
            ui.painter().rect_stroke(
                rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0_f32, border_color),
            );

            // Draw left accent bar + tactical brackets for active tab
            if is_active {
                let accent_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top()),
                    egui::pos2(rect.left() + 3.0, rect.bottom()),
                );
                ui.painter().rect_filled(accent_rect, egui::Rounding::same(6.0), accent_color);
                crate::ui::primitives::draw_cyber_corner_brackets(
                    ui,
                    rect,
                    egui::Color32::from_rgba_unmultiplied(0, 229, 255, 130),
                );
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

            // Paint live process count badge on the master Processes tab.
            if tab == crate::app_tabs::ActiveTab::Processes {
                if let Some(ref snapshot) = latest_snapshot {
                    let count = snapshot
                        .cpu
                        .as_ref()
                        .map(|cpu| cpu.process_count as u32)
                        .or_else(|| snapshot.memory.as_ref().map(|mem| mem.system_processes))
                        .unwrap_or(0);
                    let pct_pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                    ui.painter().text(
                        pct_pos,
                        egui::Align2::RIGHT_CENTER,
                        cached_count(count),
                        egui::FontId::monospace(14.0),
                        colors::ACCENT_CYAN,
                    );
                }
            }

            // Paint live RAM % badge on Memory tab at RIGHT_CENTER
            if tab == crate::app_tabs::ActiveTab::Memory {
                if let Some(ref snapshot) = latest_snapshot {
                    if let Some(ref mem) = snapshot.memory {
                        let ram_pct = mem.ram_usage_percent();
                        let pct_color = crate::theme::load_color(ram_pct);
                        let pct_text = cached_pct(ram_pct);
                        let pct_pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                        ui.painter().text(
                            pct_pos,
                            egui::Align2::RIGHT_CENTER,
                            pct_text,
                            egui::FontId::monospace(14.0),
                            pct_color,
                        );
                    }
                }
            }

            // Paint live CPU % badge on CPU tab at RIGHT_CENTER
            if tab == crate::app_tabs::ActiveTab::Cpu {
                if let Some(ref snapshot) = latest_snapshot {
                    if let Some(ref cpu) = snapshot.cpu {
                        let cpu_pct = cpu.total_pct;
                        let pct_color = crate::theme::load_color(cpu_pct);
                        let pct_text = cached_cpu_pct(cpu_pct);
                        let pct_pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                        ui.painter().text(
                            pct_pos,
                            egui::Align2::RIGHT_CENTER,
                            pct_text,
                            egui::FontId::monospace(14.0),
                            pct_color,
                        );
                    }
                }
            }

            // Paint live storage badge on Storage tab at RIGHT_CENTER
            if tab == crate::app_tabs::ActiveTab::Storage {
                if let Some(ref snapshot) = latest_snapshot {
                    let total = snapshot.storage.read_mb_s + snapshot.storage.write_mb_s;
                    let badge = cached_storage_badge(total);
                    let badge_color = if hysteresis_active(&STORAGE_ACTIVE, total) {
                        colors::ACCENT_CYAN
                    } else {
                        colors::TEXT_SECONDARY
                    };
                    let pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                    ui.painter().text(
                        pos,
                        egui::Align2::RIGHT_CENTER,
                        badge,
                        egui::FontId::monospace(14.0),
                        badge_color,
                    );
                }
            }

            // Paint live network badge on Network tab at RIGHT_CENTER
            if tab == crate::app_tabs::ActiveTab::Network {
                if let Some(ref snapshot) = latest_snapshot {
                    let total = snapshot.network.rx_mbps + snapshot.network.tx_mbps;
                    let badge = cached_network_badge(total);
                    let badge_color = if hysteresis_active(&NETWORK_ACTIVE, total) {
                        colors::ACCENT_CYAN
                    } else {
                        colors::TEXT_SECONDARY
                    };
                    let pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                    ui.painter().text(
                        pos,
                        egui::Align2::RIGHT_CENTER,
                        badge,
                        egui::FontId::monospace(14.0),
                        badge_color,
                    );
                }
            }

            // Roadmap badges for planned Stage B modules.
            let roadmap = match tab {
                crate::app_tabs::ActiveTab::System => Some("v0.5"),
                _ => None,
            };
            if let Some(label) = roadmap {
                let pos = egui::pos2(rect.right() - 12.0, rect.center().y);
                ui.painter().text(
                    pos,
                    egui::Align2::RIGHT_CENTER,
                    label,
                    egui::FontId::monospace(11.0),
                    egui::Color32::from_rgba_unmultiplied(92, 104, 122, 190),
                );
            }

            if response.clicked() {
                *active_tab = tab;
            }
            if response.has_focus() {
                let (up, down, confirm) = ui.input(|i| {
                    (
                        i.key_pressed(egui::Key::ArrowUp),
                        i.key_pressed(egui::Key::ArrowDown),
                        i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space),
                    )
                });
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

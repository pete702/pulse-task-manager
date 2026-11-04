//! Kernel metrics mini-tiles for PULSE memory HUD.

use eframe::egui;

use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};

const NA_TEXT: &str = "N/A";

/// Renders a single kernel metrics tile with fixed dimensions to prevent wrapping.
fn render_tile(
    ui: &mut egui::Ui,
    title: &str,
    main_val: &str,
    main_color: egui::Color32,
    subtitle: &str,
) {
    // Fixed dimensions to prevent height mismatch. Use the width available
    // inside the card frame so the tile never widens the parent layout.
    let size = egui::vec2(ui.available_width(), 66.0);
    ui.set_min_size(size);
    ui.set_max_size(size);
    ui.vertical_centered(|ui| {
        ui.add_space(2.0);
        ui.label(egui::RichText::new(title).monospace().size(11.0).color(colors::TEXT_SECONDARY));
        ui.add_space(3.0);
        ui.label(egui::RichText::new(main_val).monospace().size(16.0).color(main_color));
        ui.add_space(1.0);
        ui.label(egui::RichText::new(subtitle).monospace().size(9.0).color(colors::TEXT_SECONDARY));
    });
}

/// Renders the 4 kernel metrics mini-tiles (System Cache, Kernel Pool, Commit Charge, OS Activity).
pub fn render_kernel_tiles(ui: &mut egui::Ui, mem: &MemorySnapshot) {
    let valid = mem.kernel_metrics_valid;

    ui.columns(4, |cols| {
        // Tile 1: System Cache
        card_frame().show(&mut cols[0], |ui| {
            let (cache_val, cache_subtitle, cache_color) = if valid {
                let cache_pct = if mem.ram_total_bytes > 0 {
                    (mem.ram_cached_bytes() as f64 / mem.ram_total_bytes as f64) * 100.0
                } else {
                    0.0
                };
                (
                    format_bytes(mem.ram_cached_bytes()),
                    format!("OS Cache • {:.1}% of RAM", cache_pct),
                    colors::ACCENT_CYAN,
                )
            } else {
                (NA_TEXT.to_owned(), "OS Cache • N/A".to_owned(), colors::TEXT_SECONDARY)
            };
            render_tile(ui, "System Cache", &cache_val, cache_color, &cache_subtitle);
        });

        // Tile 2: Kernel Pool (using integer MiB to prevent wrapping)
        card_frame().show(&mut cols[1], |ui| {
            let (pool_val, pool_subtitle, pool_color) = if valid {
                let paged_mib = mem.kernel_paged_mib();
                let nonpaged_mib = mem.kernel_nonpaged_mib();
                (
                    format_bytes(mem.kernel_paged_bytes + mem.kernel_nonpaged_bytes),
                    format!("Paged: {:.0}M • Non-Paged: {:.0}M", paged_mib, nonpaged_mib),
                    colors::ACCENT_AMBER,
                )
            } else {
                (
                    NA_TEXT.to_owned(),
                    "Paged: N/A • Non-Paged: N/A".to_owned(),
                    colors::TEXT_SECONDARY,
                )
            };
            render_tile(ui, Key::KernelPool.t(), &pool_val, pool_color, &pool_subtitle);
        });

        // Tile 3: Commit Charge
        card_frame().show(&mut cols[2], |ui| {
            let (commit_val, commit_subtitle, commit_color) = if valid {
                let commit_pct = if mem.commit_limit_bytes > 0 {
                    (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64 * 100.0) as f32
                } else {
                    0.0
                };
                (
                    format_bytes(mem.commit_total_bytes),
                    format!("Limit: {} ({:.1}%)", format_bytes(mem.commit_limit_bytes), commit_pct),
                    colors::ACCENT_PURPLE,
                )
            } else {
                (NA_TEXT.to_owned(), "Limit: N/A (N/A%)".to_owned(), colors::TEXT_SECONDARY)
            };
            render_tile(ui, Key::CommitCharge.t(), &commit_val, commit_color, &commit_subtitle);
        });

        // Tile 4: OS Activity (2-level structure like other tiles)
        card_frame().show(&mut cols[3], |ui| {
            let (activity_val, activity_subtitle, activity_color) = if valid {
                (
                    format!("{} Procs", mem.system_processes),
                    format!("{} Threads • {} Handles", mem.system_threads, mem.system_handles),
                    colors::TEXT_PRIMARY,
                )
            } else {
                (
                    "N/A Procs".to_owned(),
                    "N/A Threads • N/A Handles".to_owned(),
                    colors::TEXT_SECONDARY,
                )
            };
            render_tile(
                ui,
                Key::OsActivityTitle.t(),
                &activity_val,
                activity_color,
                &activity_subtitle,
            );
        });
    });
}

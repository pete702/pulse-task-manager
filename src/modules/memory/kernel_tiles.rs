//! Kernel metrics mini-tiles for PULSE memory HUD.

use eframe::egui;

use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};

/// Renders a single kernel metrics tile with fixed dimensions to prevent wrapping.
fn render_tile(
    ui: &mut egui::Ui,
    width: f32,
    title: &str,
    main_val: &str,
    main_color: egui::Color32,
    subtitle: &str,
) {
    // Fixed dimensions to prevent height mismatch
    ui.set_min_size(egui::vec2(width, 66.0));
    ui.set_max_size(egui::vec2(width, 66.0));
    ui.vertical_centered(|ui| {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(title)
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );
        ui.add_space(3.0);
        ui.label(
            egui::RichText::new(main_val)
                .monospace()
                .size(16.0)
                .color(main_color),
        );
        ui.add_space(1.0);
        ui.label(
            egui::RichText::new(subtitle)
                .monospace()
                .size(9.0)
                .color(colors::TEXT_SECONDARY),
        );
    });
}

/// Renders the 4 kernel metrics mini-tiles (Standby Cache, Kernel Pool, Commit Charge, OS Activity).
pub fn render_kernel_tiles(ui: &mut egui::Ui, mem: &MemorySnapshot) {
    let width = ui.available_width() / 4.0 - 4.0; // Account for column spacing

    ui.columns(4, |cols| {
        // Tile 1: Standby Cache
        card_frame().show(&mut cols[0], |ui| {
            let cache_pct = if mem.ram_total_bytes > 0 {
                (mem.ram_cached_bytes() as f64 / mem.ram_total_bytes as f64) * 100.0
            } else {
                0.0
            };
            render_tile(
                ui,
                width,
                "Standby Cache",
                &format_bytes(mem.ram_cached_bytes()),
                colors::ACCENT_CYAN,
                &format!("OS Cache • {:.1}% of RAM", cache_pct),
            );
        });

        // Tile 2: Kernel Pool (using integer MiB to prevent wrapping)
        card_frame().show(&mut cols[1], |ui| {
            let paged_mib = mem.kernel_paged_mib();
            let nonpaged_mib = mem.kernel_nonpaged_mib();
            render_tile(
                ui,
                width,
                Key::KernelPool.t(),
                &format_bytes(mem.kernel_paged_bytes + mem.kernel_nonpaged_bytes),
                colors::ACCENT_AMBER,
                &format!("Paged: {:.0}M • Non-Paged: {:.0}M", paged_mib, nonpaged_mib),
            );
        });

        // Tile 3: Commit Charge
        card_frame().show(&mut cols[2], |ui| {
            let commit_pct = if mem.commit_limit_bytes > 0 {
                (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64 * 100.0) as f32
            } else {
                0.0
            };
            render_tile(
                ui,
                width,
                Key::CommitCharge.t(),
                &format_bytes(mem.commit_total_bytes),
                colors::ACCENT_PURPLE,
                &format!(
                    "Limit: {} ({:.1}%)",
                    format_bytes(mem.commit_limit_bytes),
                    commit_pct
                ),
            );
        });

        // Tile 4: OS Activity (2-level structure like other tiles)
        card_frame().show(&mut cols[3], |ui| {
            render_tile(
                ui,
                width,
                Key::OsActivityTitle.t(),
                &format!("{} Procs", mem.system_processes),
                colors::TEXT_PRIMARY,
                &format!(
                    "{} Threads • {} Handles",
                    mem.system_threads, mem.system_handles
                ),
            );
        });
    });
}

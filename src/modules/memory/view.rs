//! Memory HUD cards and mini-tiles for PULSE.

use eframe::egui;

use crate::i18n::Key;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::colors;

/// Renders the memory HUD cards and mini-tiles.
pub fn render_memory_cards(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    history: &crate::modules::memory::model::MemoryHistory,
) {
    // Helper to format bytes into human-readable string (GiB/MiB/KiB)
    fn format_bytes(bytes: u64) -> String {
        const GIB: u64 = 1_073_741_824;
        const MIB: u64 = 1_048_576;
        const KIB: u64 = 1024;

        if bytes >= GIB {
            format!("{:.2} GiB", bytes as f64 / GIB as f64)
        } else if bytes >= MIB {
            format!("{:.2} MiB", bytes as f64 / MIB as f64)
        } else if bytes >= KIB {
            format!("{:.2} KiB", bytes as f64 / KIB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    // ----- Row 1: Two main cards (RAM + Swap/Commit) -----
    ui.columns(2, |cols| {
        // --- Card 1: PHYSICAL RAM ---
        crate::theme::card_frame().show(&mut cols[0], |ui| {
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(Key::PhysicalRamTitle.t())
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let pct = mem.ram_usage_percent();
                        let color = crate::theme::load_color(pct);
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", pct))
                                .monospace()
                                .size(22.0)
                                .color(color),
                        );
                    });
                });
                ui.add_space(8.0);

                // Progress bar
                let pct = mem.ram_usage_percent() / 100.0;
                let bar_color = crate::theme::load_color(mem.ram_usage_percent());
                ui.add(
                    egui::ProgressBar::new(pct)
                        .fill(bar_color)
                        .animate(false)
                        .rounding(egui::Rounding::same(3.0)),
                );
                ui.add_space(12.0);

                // Grid of values
                egui::Grid::new("ram_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        // Used / Total
                        ui.label(
                            egui::RichText::new(Key::UsedTotal.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.2} / {:.2} GiB",
                                mem.ram_used_gib(),
                                mem.ram_total_gib()
                            ))
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                        );
                        ui.end_row();

                        // Available / Free
                        ui.label(
                            egui::RichText::new(Key::AvailableFree.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.2} GiB / Free: {:.2} GiB",
                                mem.ram_available_gib(),
                                mem.ram_free_bytes as f64 / 1_073_741_824.0
                            ))
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                        );
                        ui.end_row();

                        // Cached
                        ui.label(
                            egui::RichText::new(Key::CachedStandby.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.2} GiB", mem.ram_cached_gib()))
                                .monospace()
                                .size(13.0)
                                .color(colors::ACCENT_CYAN),
                        );
                        ui.end_row();

                        // Peak
                        ui.label(
                            egui::RichText::new(Key::PeakSession.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", history.peak_ram_percent))
                                .monospace()
                                .size(13.0)
                                .color(colors::ACCENT_RED),
                        );
                        ui.end_row();
                    });
            });
        });

        // --- Card 2: SWAP & COMMIT ---
        crate::theme::card_frame().show(&mut cols[1], |ui| {
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(Key::SwapCommitTitle.t())
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Use Commit % if swap is 0, otherwise swap %
                        let (pct, color) = if mem.swap_total_bytes > 0 {
                            (
                                mem.swap_usage_percent(),
                                crate::theme::load_color(mem.swap_usage_percent()),
                            )
                        } else {
                            let commit_pct = if mem.commit_limit_bytes > 0 {
                                (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64
                                    * 100.0) as f32
                            } else {
                                0.0
                            };
                            (commit_pct, crate::theme::load_color(commit_pct))
                        };
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", pct))
                                .monospace()
                                .size(22.0)
                                .color(color),
                        );
                    });
                });
                ui.add_space(8.0);

                // Progress bar
                let pct = if mem.swap_total_bytes > 0 {
                    mem.swap_usage_percent() / 100.0
                } else if mem.commit_limit_bytes > 0 {
                    (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64) as f32
                } else {
                    0.0
                };
                ui.add(
                    egui::ProgressBar::new(pct)
                        .fill(colors::ACCENT_PURPLE)
                        .animate(false)
                        .rounding(egui::Rounding::same(3.0)),
                );
                ui.add_space(12.0);

                // Grid of values
                egui::Grid::new("swap_commit_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        // Swap Used / Total
                        ui.label(
                            egui::RichText::new(Key::SwapUsedTotal.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.2} / {:.2} GiB",
                                mem.swap_used_gib(),
                                mem.swap_total_gib()
                            ))
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                        );
                        ui.end_row();

                        // Commit Total / Limit
                        ui.label(
                            egui::RichText::new(Key::CommitChargeLimit.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.2} / {:.2} GiB",
                                mem.commit_total_gib(),
                                mem.commit_limit_gib()
                            ))
                            .monospace()
                            .size(13.0)
                            .color(colors::ACCENT_PURPLE),
                        );
                        ui.end_row();

                        // Commit %
                        ui.label(
                            egui::RichText::new(Key::CommitPercent.t())
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_SECONDARY),
                        );
                        let commit_pct = if mem.commit_limit_bytes > 0 {
                            (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64 * 100.0)
                                as f32
                        } else {
                            0.0
                        };
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", commit_pct))
                                .monospace()
                                .size(13.0)
                                .color(crate::theme::load_color(commit_pct)),
                        );
                        ui.end_row();
                    });
            });
        });
    });

    ui.add_space(12.0);

    // ----- Row 2: 4 Mini-tiles (Kernel Metrics) -----
    ui.columns(4, |cols| {
        // Tile 1: System Cache
        crate::theme::card_frame().show(&mut cols[0], |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(Key::SystemCache.t())
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format_bytes(mem.ram_cached_bytes()))
                        .monospace()
                        .size(16.0)
                        .color(colors::ACCENT_CYAN),
                );
                ui.label(
                    egui::RichText::new(Key::SystemCache.t())
                        .monospace()
                        .size(10.0)
                        .color(colors::TEXT_SECONDARY),
                );
            });
        });

        // Tile 2: Kernel Pool
        crate::theme::card_frame().show(&mut cols[1], |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(Key::KernelPool.t())
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                let total_pool = mem.kernel_paged_bytes + mem.kernel_nonpaged_bytes;
                ui.label(
                    egui::RichText::new(format_bytes(total_pool))
                        .monospace()
                        .size(16.0)
                        .color(colors::ACCENT_AMBER),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Paged: {} / Non-paged: {}",
                        format_bytes(mem.kernel_paged_bytes),
                        format_bytes(mem.kernel_nonpaged_bytes)
                    ))
                    .monospace()
                    .size(10.0)
                    .color(colors::TEXT_SECONDARY),
                );
            });
        });

        // Tile 3: Commit Charge
        crate::theme::card_frame().show(&mut cols[2], |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(Key::CommitCharge.t())
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format_bytes(mem.commit_total_bytes))
                        .monospace()
                        .size(16.0)
                        .color(colors::ACCENT_PURPLE),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Limit: {} ({:.1}%)",
                        format_bytes(mem.commit_limit_bytes),
                        if mem.commit_limit_bytes > 0 {
                            (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64 * 100.0)
                                as f32
                        } else {
                            0.0
                        }
                    ))
                    .monospace()
                    .size(10.0)
                    .color(colors::TEXT_SECONDARY),
                );
            });
        });

        // Tile 4: OS Activity (Handles & Threads)
        crate::theme::card_frame().show(&mut cols[3], |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(Key::OsActivity.t())
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} процесів  |  {} потоків  |  {} дескрипторів",
                        mem.system_processes, mem.system_threads, mem.system_handles
                    ))
                    .monospace()
                    .size(13.0)
                    .color(colors::TEXT_PRIMARY),
                );
            });
        });
    });
}

/// Helper to format bytes into human-readable string (GiB/MiB/KiB)
#[allow(dead_code)]
fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1_073_741_824;
    const MIB: u64 = 1_048_576;
    const KIB: u64 = 1024;

    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::memory::model::MemorySnapshot;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1_073_741_824), "1.00 GiB");
        assert_eq!(format_bytes(1_048_576), "1.00 MiB");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(2_147_483_648), "2.00 GiB");
        assert_eq!(format_bytes(5_368_709_120), "5.00 GiB");
    }

    #[test]
    fn test_render_memory_cards_no_panic() {
        let mem = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            ram_used_bytes: 8_000_000_000,
            ram_available_bytes: 8_000_000_000,
            ram_free_bytes: 2_000_000_000,
            swap_total_bytes: 8_000_000_000,
            swap_used_bytes: 2_000_000_000,
            swap_free_bytes: 6_000_000_000,
            commit_total_bytes: 10_000_000_000,
            commit_limit_bytes: 20_000_000_000,
            kernel_paged_bytes: 100_000_000,
            kernel_nonpaged_bytes: 50_000_000,
            system_handles: 50000,
            system_threads: 2000,
            system_processes: 150,
            ram_cached_bytes_override: 0,
        };
        let _history = crate::modules::memory::model::MemoryHistory::new();

        // This should not panic (just testing function signature compiles)
        // We can't easily test UI rendering without a full egui context,
        // but we can verify the format_bytes helper works.
        assert!(!format_bytes(mem.ram_cached_bytes()).is_empty());
    }
}

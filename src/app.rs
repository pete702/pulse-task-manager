//! Головний стан додатку PULSE: UI, обробка телеметрії, вкладки.

use std::sync::mpsc;

use eframe::egui;

use crate::core::{spawn_telemetry_worker, SystemSnapshot};
use crate::theme::{apply_obsidian_theme, card_frame};

/// Активна вкладка в лівій панелі.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Вкладка пам'яті (поки єдина).
    Memory,
}

impl ActiveTab {
    /// Повертає всі доступні вкладки.
    pub const fn all() -> &'static [ActiveTab] {
        &[ActiveTab::Memory]
    }

    /// Текст кнопки для вкладки.
    pub fn label(self) -> &'static str {
        match self {
            ActiveTab::Memory => "🧠 Пам'ять",
        }
    }
}

/// Головний стан додатку.
pub struct PulseApp {
    /// Отримувач снепшотів з фонового потоку.
    snapshot_rx: mpsc::Receiver<SystemSnapshot>,
    /// Останній отриманий снепшот.
    latest_snapshot: Option<SystemSnapshot>,
    /// Поточна активна вкладка.
    active_tab: ActiveTab,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            active_tab: ActiveTab::Memory,
        }
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Застосовуємо тему
        apply_obsidian_theme(ctx);

        // Неблокуюче читання снапшотів з каналу
        while let Ok(snapshot) = self.snapshot_rx.try_recv() {
            self.latest_snapshot = Some(snapshot);
        }

        // Верхня панель: заголовок, версія, uptime
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("⚡ PULSE // TELEMETRY")
                        .monospace()
                        .size(18.0),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .monospace()
                        .size(14.0),
                );
                ui.separator();

                if let Some(snapshot) = self.latest_snapshot {
                    let uptime_str = format_uptime(snapshot.uptime_secs);
                    ui.label(
                        egui::RichText::new(format!("Uptime: {}", uptime_str))
                            .monospace()
                            .size(14.0),
                    );
                } else {
                    ui.label(egui::RichText::new("Uptime: --").monospace().size(14.0));
                }
            });
        });

        // Ліва панель: навігація вкладок
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(180.0)
            .max_width(180.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    for tab in ActiveTab::all() {
                        let is_active = self.active_tab == *tab;
                        let btn = ui.selectable_label(is_active, tab.label());
                        if btn.clicked() {
                            self.active_tab = *tab;
                        }
                    }
                });
            });

        // Центральна панель: контент залежно від вкладки
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);
            card_frame().show(ui, |ui| {
                ui.label("Ядро телеметрії активне. Очікування моделі пам'яті (v0.0.3)...");
            });
        });

        // Запитуємо перемалювання для оновлення uptime
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

/// Форматує uptime у секундах у читабельний рядок (дні, години, хвилини, секунди).
fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}д", days));
    }
    if hours > 0 || days > 0 {
        parts.push(format!("{}г", hours));
    }
    if mins > 0 || hours > 0 || days > 0 {
        parts.push(format!("{}хв", mins));
    }
    parts.push(format!("{}с", secs));

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(0), "0с");
        assert_eq!(format_uptime(30), "30с");
        assert_eq!(format_uptime(59), "59с");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(60), "1хв 0с");
        assert_eq!(format_uptime(90), "1хв 30с");
        assert_eq!(format_uptime(3599), "59хв 59с");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3600), "1г 0хв 0с");
        assert_eq!(format_uptime(7200), "2г 0хв 0с");
        assert_eq!(format_uptime(86399), "23г 59хв 59с");
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(86400), "1д 0г 0хв 0с");
        assert_eq!(format_uptime(172800), "2д 0г 0хв 0с");
        assert_eq!(format_uptime(90000), "1д 1г 0хв 0с");
    }

    #[test]
    fn test_active_tab_all() {
        let tabs = ActiveTab::all();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0], ActiveTab::Memory);
    }

    #[test]
    fn test_active_tab_label() {
        assert_eq!(ActiveTab::Memory.label(), "🧠 Пам'ять");
    }
}

//! UI Chrome: Obsidian Cyan top bar, sidebar toggle and compact tab bar.
use crate::app_cmd::AppCommand;
use crate::app_tabs::ActiveTab;
use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_short, Key};
use crate::platform::hotkey;
use crate::theme::colors;
use crate::ui::primitives::{pill_badge, pill_button, pill_button_colored, pill_toggle};
pub use crate::ui::status_bar::render_status_bar;
use crate::util::format_uptime;
use eframe::egui;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

fn cached_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION
        .get_or_init(|| format!("{}{}", Key::VersionPrefix.t(), env!("CARGO_PKG_VERSION")))
        .as_str()
}

static LAST_SELF_RAM_X10: AtomicU32 = AtomicU32::new(426);

fn cached_self_ram_mb(fresh: Option<f32>) -> f32 {
    if let Some(v) = fresh {
        let clean = if v.is_finite() && v > 0.0 && v < 1_000_000.0 { v } else { 0.0 };
        if clean > 0.0 {
            LAST_SELF_RAM_X10.store((clean * 10.0).round() as u32, Ordering::Relaxed);
        }
    }
    LAST_SELF_RAM_X10.load(Ordering::Relaxed) as f32 / 10.0
}

fn cached_format_uptime(uptime_secs: u64) -> Arc<str> {
    static CACHE: OnceLock<Mutex<(u64, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != uptime_secs {
        cache.0 = uptime_secs;
        cache.1 = Arc::from(format_uptime(uptime_secs));
    }
    Arc::clone(&cache.1)
}

/// Far-left 28 px `<<` / `>>` toggle pill drawn before the Pause control.
pub fn render_sidebar_toggle(ui: &mut egui::Ui, sidebar_hidden: &mut bool) -> bool {
    let label = if *sidebar_hidden { ">>" } else { "<<" };
    let button = egui::Button::new(
        egui::RichText::new(label)
            .font(egui::FontId::proportional(12.0))
            .color(colors::ACCENT_CYAN),
    )
    .rounding(egui::Rounding::same(colors::PILL_RADIUS))
    .min_size(egui::vec2(28.0_f32, 18.0_f32));
    let response = ui
        .add(button)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Toggle sidebar (Ctrl+B)");
    if response.clicked() {
        *sidebar_hidden = !*sidebar_hidden;
        return true;
    }
    false
}

/// Renders the 40 px top panel as a row of rounded pills.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
    close_to_tray: &mut bool,
    cmds: &mut Vec<AppCommand>,
) {
    let bar_w = ui.available_width();
    let title = Key::AppTitle.t();
    let title = title.split(" //").next().unwrap_or(title);
    ui.heading(
        egui::RichText::new(title)
            .font(egui::FontId::proportional(16.0))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(4.0);
    pill_badge(ui, cached_version(), colors::TEXT_SECONDARY);

    let uptime_str = latest_snapshot
        .as_ref()
        .map(|s| cached_format_uptime(s.uptime_secs))
        .unwrap_or_else(|| Arc::from("--"));
    let (cpu_pct, ram_pct) = match latest_snapshot.as_ref().and_then(|s| {
        s.cpu.as_ref().map(|c| (c.total_pct, s.memory.as_ref().map(|m| m.ram_usage_percent())))
    }) {
        Some((cpu, Some(ram))) => {
            let cpu = if cpu.is_finite() { cpu.clamp(0.0, 100.0) } else { 0.0 };
            let ram = if ram.is_finite() { ram.clamp(0.0, 100.0) } else { 0.0 };
            (cpu, ram)
        }
        _ => (0.0, 0.0),
    };
    let ram_mb = cached_self_ram_mb(latest_snapshot.as_ref().map(|s| s.pulse_ram_mb));
    let slot_w = if bar_w >= 1280.0 { 310.0 } else { 186.0 };
    if bar_w >= 1080.0 {
        let readout = if bar_w >= 1280.0 {
            format!("Uptime {} | CPU {:>3.0}% | RAM {:>3.0}%", uptime_str, cpu_pct, ram_pct)
        } else {
            format!("CPU {:>3.0}% | RAM {:>3.0}%", cpu_pct, ram_pct)
        };
        ui.allocate_ui_with_layout(
            egui::vec2(slot_w, 22.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let label = egui::Label::new(
                    egui::RichText::new(readout)
                        .font(egui::FontId::monospace(12.0))
                        .color(colors::ACCENT_CYAN),
                )
                .wrap(false)
                .truncate(true);
                ui.add(label).on_hover_text(format!(
                    "PULSE: {ram_mb:.1} MB | UI: {last_ui_ms:.2} ms | OS: {} | {}: {uptime_str}",
                    os_locale_short(),
                    Key::UptimeLabel.t()
                ));
            },
        );
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if bar_w >= 720.0 {
            ui.allocate_ui_with_layout(
                egui::vec2(130.0, 22.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| render_health_pill(ui, latest_snapshot),
            );
        }
        if pill_button(ui, "Exit").clicked() {
            crate::platform::tray::set_allow_real_close(true);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let mut hud_on = hotkey::is_mini_hud_active();
        if pill_toggle(ui, "Mini-HUD", &mut hud_on).clicked() {
            hotkey::set_mini_hud_active(hud_on);
            ui.ctx().request_repaint();
        }
        let mut hk_on = hotkey::is_global_hotkey_enabled();
        let hk_label = format!("HK: {}", hotkey::hotkey_status_label());
        if pill_toggle(ui, &hk_label, &mut hk_on).on_hover_text(hotkey::HOTKEY_TOOLTIP).clicked() {
            hotkey::set_global_hotkey_enabled(hk_on);
            crate::core::persist::save_hotkey_enabled(hk_on);
            hotkey::request_rebind();
            ui.ctx().request_repaint();
        }
        if pill_button(ui, "Tray").clicked() {
            cmds.push(AppCommand::HideToTray);
        }
        if pill_button(ui, "Ctrl+K").on_hover_text("Open Command Palette (Ctrl+K)").clicked() {
            crate::ui::command_palette::toggle();
        }
        let _ = pill_toggle(ui, "Tray on X", close_to_tray);
    });
}

fn render_health_pill(ui: &mut egui::Ui, snapshot: &Option<Arc<SystemSnapshot>>) {
    let Some(snapshot) = snapshot else { return };
    use crate::ui::three_walls::WallState;
    let wall = crate::ui::three_walls::evaluate(snapshot);
    let bottleneck = snapshot.cpu.as_ref().is_some_and(|c| c.single_thread_bottleneck());
    let (storage_low, storage_summary) =
        crate::modules::storage::model::low_space_alert(&snapshot.storage.volumes);
    let net_bad = matches!(
        snapshot.network.grade,
        crate::modules::network::model::ConnectionGrade::Offline
            | crate::modules::network::model::ConnectionGrade::PacketLoss
    );
    let (text, color) = match wall {
        WallState::MemoryWall => ("MEMORY WALL", colors::ACCENT_RED),
        _ if storage_low => ("STORAGE LOW", colors::ACCENT_RED),
        _ if net_bad => ("NO NETWORK", colors::ACCENT_RED),
        WallState::Elevated => ("HEALTH: ELEVATED", colors::ACCENT_AMBER),
        WallState::Unconfirmed => ("HEALTH: INIT", colors::TEXT_SECONDARY),
        WallState::Optimal if bottleneck => ("BOTTLENECK", colors::ACCENT_AMBER),
        WallState::Optimal => ("HEALTH: NOMINAL", colors::EMERALD),
    };
    let response = pill_badge(ui, text, color);
    if response.hovered() {
        let mut tooltip = snapshot.memory.as_ref().map_or_else(
            || "Awaiting memory telemetry".to_owned(),
            |m| {
                let cpu = snapshot.cpu.as_ref().map_or(0.0, |c| c.total_pct);
                let cpu = if cpu.is_finite() { cpu.clamp(0.0, 100.0) } else { 0.0 };
                let hot = snapshot.cpu.as_ref().map_or(0.0, |c| c.hottest_core_pct);
                let hot = if hot.is_finite() { hot.clamp(0.0, 100.0) } else { 0.0 };
                format!(
                    "Three Walls: {}\nRAM: {:.1}% Commit: {:.1}%\nCPU: {:.1}% Hottest: {:.1}%",
                    wall.label(),
                    m.ram_usage_percent(),
                    m.commit_charge_pct(),
                    cpu,
                    hot
                )
            },
        );
        if storage_low && !storage_summary.is_empty() {
            tooltip.push('\n');
            tooltip.push_str(&storage_summary);
        }
        response.on_hover_text(tooltip);
    }
}

/// Compact 28 px pill bar shown when the left sidebar is hidden.
pub fn render_compact_tabs(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    cmds: &mut Vec<AppCommand>,
) {
    let old = *active_tab;
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        for tab in ActiveTab::all_ordered() {
            let label = tab.short_label();
            let color =
                if *active_tab == tab { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
            if pill_button_colored(ui, label, color).on_hover_text(label).clicked() {
                *active_tab = tab;
            }
        }
    });
    if *active_tab != old {
        cmds.push(AppCommand::SetTab(*active_tab));
    }
}

#[cfg(test)]
#[path = "chrome_tests.rs"]
mod chrome_tests;

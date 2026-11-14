//! Memory module for PULSE: RAM/swap collector, HUD view, chart, and safety helpers.

pub mod chart_axis;
pub mod chart_chrome;
pub mod chart_hud;
pub mod chart_view;
pub mod collector;
pub mod constants;
pub mod delta_tracker;
pub mod delta_tracker_tests;
pub mod formatters;
pub mod history;
pub mod history_tests;
pub mod model;
pub mod model_conversions;
pub mod page;
pub mod process_spike;
pub mod top_cards;
pub mod view;

pub use crate::platform::handle::query_creation_time_by_pid;
pub use crate::platform::win32_api::query_process_exe_path;

#[cfg(test)]
mod chart_tests;

//! Memory module for PULSE: pure math model, sysinfo collector, HUD view,
//! multi-horizon chart, and the virtualized process table.

pub mod action_bar;
pub mod action_bar_tests;
pub mod actions;
pub mod chart_view;
pub mod collector;
pub mod composition_bar;
pub mod delta_tracker;
pub mod formatters;
pub mod history;
pub mod kernel_tiles;
pub mod model;
pub mod model_conversions;
pub mod page;
pub mod process_model;
pub mod process_model_tests;
pub mod ram_card;
pub mod safe_handle;
pub mod swap_commit_card;
mod table_body;
pub mod table_body_tests;
pub mod table_view;
pub mod table_view_tests;
mod table_widgets;
pub mod view;
pub mod view_cards;
pub mod win32_api;

#[cfg(test)]
mod actions_tests;

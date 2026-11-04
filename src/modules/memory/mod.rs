//! Memory module for PULSE: pure math model, sysinfo collector, HUD view,
//! multi-horizon chart, and the virtualized process table.

pub mod action_bar;
pub mod action_bar_status;
pub mod action_bar_tests;
pub mod action_bar_view;
pub mod actions;
pub mod chart_axis;
pub mod chart_chrome;
pub mod chart_view;
pub mod collector;
pub mod composition_bar;
pub mod delta_tracker;
pub mod delta_tracker_tests;
pub mod formatters;
pub mod history;
pub mod kernel_tiles;
pub mod model;
pub mod model_conversions;
pub mod page;
pub mod process_model;
pub mod process_model_tests;
pub mod process_target;
pub mod protected_lists;
pub mod ram_card;
pub mod safe_handle;
pub mod selection;
pub mod selection_tests;
pub mod swap_commit_card;
mod table_body;
pub mod table_body_tests;
mod table_cache;
pub mod table_selection;
pub mod table_selection_tests;
pub mod table_state;
pub mod table_view;
pub mod table_view_tests;
mod table_widgets;
pub mod view;
pub mod view_cards;
pub mod win32_api;

#[cfg(test)]
mod actions_tests;

#[cfg(test)]
mod chart_tests;

#[cfg(test)]
mod table_selection_cache_tests;

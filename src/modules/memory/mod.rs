//! Memory module for PULSE: pure math model, sysinfo collector, HUD view,
//! multi-horizon chart, and the virtualized process table.

pub mod chart_view;
pub mod collector;
pub mod history;
pub mod kernel_tiles;
pub mod model;
pub mod model_conversions;
pub mod page;
pub mod process_model;
pub mod ram_card;
pub mod swap_commit_card;
mod table_body;
pub mod table_view;
mod table_widgets;
pub mod view;
pub mod view_cards;
pub mod win32_api;

#[cfg(test)]
mod chart_tests;
#[cfg(test)]
mod collector_tests;
#[cfg(test)]
mod model_history_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod process_tests;
#[cfg(test)]
mod view_tests;

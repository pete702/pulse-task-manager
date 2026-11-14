//! Platform abstraction layer for PULSE Win32 helpers.
//! Strictly zero egui imports; localized unsafe blocks only.

pub mod handle;

pub mod actions;
#[cfg(test)]
mod actions_tests;
#[cfg(test)]
mod handle_tests;
pub mod location;
pub mod protected;
pub mod qos;
pub mod sys_metrics;
pub mod target;
pub mod trim;
pub mod win32_api;
pub mod win32_cpu;

// No blanket re-exports (B20-B-09). Use concrete submodules.

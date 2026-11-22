//! Platform abstraction layer for PULSE Win32 helpers.
//! Strictly zero egui imports; localized unsafe blocks only.

pub mod actions;
#[cfg(test)]
mod actions_tests;
pub mod dxgi_gpu;
pub mod file_lock;
pub mod handle;
#[cfg(test)]
mod handle_tests;
pub mod hotkey;
pub mod hybrid_cores;
pub mod instance;
pub mod location;
pub mod pdh_gpu;
pub mod pdh_gpu_counters;
pub mod pick_window;
pub mod protected;
pub mod qos;
pub mod services_startup;
pub mod suspend;
#[cfg(test)]
mod suspend_tests;
pub mod sys_metrics;
pub mod target;
pub mod tray;
pub mod trim;
pub mod win32_api;
pub mod win32_cpu;

// No blanket re-exports (B20-B-09). Use concrete submodules.

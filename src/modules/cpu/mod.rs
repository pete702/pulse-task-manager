//! CPU module -- 1920-sample (15m @ 2 Hz) dual-series history, process telemetry, and UI.

pub mod bottleneck_card;
pub mod chart_axis;
pub mod chart_hud;
pub mod chart_view;
pub mod collector;
pub mod collector_cpu;
pub mod collector_eco;
pub mod core_grid;
pub mod exe_cache;
pub mod history;
pub mod kill_modal;
pub mod model;
pub mod page;
pub mod proc_meta;

pub mod quick_filter;
pub mod table;
pub mod table_body;
pub mod table_body_hover;
pub mod table_cols;
pub mod table_groups;
pub mod table_menu;
pub mod table_menu_actions;
pub mod table_menu_affinity;
pub mod table_menu_proc;
pub mod table_rows;
pub mod table_sort;
pub mod table_state;
pub mod table_tools;
pub mod table_tree;
pub mod top_cards;
pub mod total_load_card;

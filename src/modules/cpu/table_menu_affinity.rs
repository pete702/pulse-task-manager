//! Affinity menu action: dispatches verified pinning to the persistent MenuJob worker.
use crate::modules::cpu::table_menu::GroupTarget;
use crate::modules::cpu::table_state::CpuTableState;
use crate::platform::hybrid_cores::{AffinityMode, AffinityTarget};
use std::time::Instant;

pub(crate) fn spawn_affinity(
    state: &mut CpuTableState,
    targets: Vec<GroupTarget>,
    p_mask: usize,
    e_mask: usize,
    mode: AffinityMode,
) {
    let items: Vec<AffinityTarget> = targets
        .into_iter()
        .map(|t| AffinityTarget { pid: t.pid, start_time: t.start_time, exe_path: t.exe_path })
        .collect();
    state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
    state.spawn_menu_action(move || {
        let message =
            crate::platform::hybrid_cores::apply_affinity_to_targets(&items, p_mask, e_mask, mode)
                .1;
        (message, None, Vec::new())
    });
}

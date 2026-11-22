//! Suspend/Resume menu actions extracted from the CPU row menu.
use crate::modules::cpu::table_menu::GroupTarget;
use crate::modules::cpu::table_state::CpuTableState;
use std::time::Instant;

fn spawn_suspend_resume(
    state: &mut CpuTableState,
    targets: Vec<GroupTarget>,
    name: String,
    resume: bool,
) {
    let total = targets.len();
    state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
    state.spawn_menu_action(move || {
        let (mut ok, mut last) = (0, String::new());
        for t in targets {
            let path = t.exe_path.as_deref().unwrap_or("");
            let r = if resume {
                crate::platform::suspend::resume_process_verified(
                    t.pid,
                    t.start_time,
                    path,
                    &t.name,
                )
            } else {
                crate::platform::suspend::suspend_process_verified(
                    t.pid,
                    t.start_time,
                    path,
                    &t.name,
                )
            };
            ok += usize::from(r.ok);
            last = if r.ok { last } else { r.message };
        }
        let m = if total == 1 {
            if ok == 1 {
                let done = if resume { "Process resumed" } else { "Process suspended" };
                done.to_owned()
            } else if resume {
                format!("Resume blocked: {last}")
            } else {
                format!("Suspend blocked: {last}")
            }
        } else if resume {
            format!("Resumed {ok}/{total} processes in {name}")
        } else {
            format!("Suspended {ok}/{total} processes in {name}")
        };
        (m, None, Vec::new())
    });
}

pub(crate) fn spawn_suspend(state: &mut CpuTableState, targets: Vec<GroupTarget>, name: String) {
    spawn_suspend_resume(state, targets, name, false);
}

pub(crate) fn spawn_resume(state: &mut CpuTableState, targets: Vec<GroupTarget>, name: String) {
    spawn_suspend_resume(state, targets, name, true);
}

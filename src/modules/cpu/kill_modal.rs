//! Kill confirmation modal helpers and protected PID guards.

#[cfg(test)]
#[derive(Debug, Clone, Default)]
pub struct KillModalState {
    pub open: bool,
    pub pids: Vec<u32>,
    pub label: String,
}

pub(crate) fn is_protected_pid(pid: u32) -> bool {
    pid == 0 || pid == 4 || pid == std::process::id()
}

#[cfg(test)]
pub fn guarded_terminate_targets(pids: &[u32]) -> (Vec<u32>, usize) {
    let mut allowed = Vec::new();
    let mut skipped = 0usize;
    for &pid in pids {
        if is_protected_pid(pid) {
            skipped += 1;
        } else {
            allowed.push(pid);
        }
    }
    (allowed, skipped)
}

pub fn termination_status(
    ok: usize,
    total: usize,
    skipped_protected: usize,
    last_err: &str,
) -> String {
    if skipped_protected > 0 && ok == 0 && total == skipped_protected {
        return "Skipped protected/self process".to_owned();
    }
    if total <= 1 {
        if ok == 1 {
            "Task ended".to_owned()
        } else if skipped_protected > 0 {
            "Skipped protected/self process".to_owned()
        } else if last_err.is_empty() {
            "End blocked".to_owned()
        } else {
            format!("End blocked: {last_err}")
        }
    } else if skipped_protected > 0 {
        format!("Terminated {ok}/{total} (skipped {skipped_protected} protected/self)")
    } else {
        format!("Terminated {ok}/{total} processes")
    }
}

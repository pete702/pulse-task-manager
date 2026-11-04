//! Stable process identity passed to verified mutating actions.

use crate::modules::memory::model::RawProcessRow;

/// A process identity that survives PID reuse checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessTarget {
    pub pid: u32,
    pub creation_time: Option<u64>,
    pub name: String,
    pub exe_path: Option<String>,
}

impl ProcessTarget {
    pub fn from_raw(row: &RawProcessRow) -> Self {
        Self {
            pid: row.pid,
            creation_time: row.creation_time,
            name: row.name.clone(),
            exe_path: row.exe_path.clone(),
        }
    }
}

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
        let exe_path = row.exe_path.as_ref().map(|s| s.to_string()).or_else(|| {
            let handle = crate::modules::memory::safe_handle::SafeHandle::open_process(row.pid)?;
            crate::modules::memory::safe_handle::query_process_image_path(&handle)
        });
        Self {
            pid: row.pid,
            creation_time: row.creation_time,
            name: row.name.to_string(),
            exe_path,
        }
    }
}

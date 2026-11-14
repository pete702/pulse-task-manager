//! Stable process identity passed to verified mutating actions.

/// A process identity that survives PID reuse checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessTarget {
    pub pid: u32,
    pub creation_time: Option<u64>,
    pub name: String,
    pub exe_path: Option<String>,
}

impl ProcessTarget {}

//! Unit conversion helpers for memory model.

use crate::modules::memory::model::MemorySnapshot;

impl MemorySnapshot {
    /// Converts bytes to gibibytes (GiB) as `f64`.
    ///
    /// 1 GiB = 1,073,741,824 bytes (2^30).
    #[inline]
    pub fn bytes_to_gib(bytes: u64) -> f64 {
        bytes as f64 / 1_073_741_824.0
    }

    /// Converts bytes to mebibytes (MiB) as `f64`.
    ///
    /// 1 MiB = 1,048,576 bytes (2^20).
    #[allow(dead_code)]
    #[inline]
    pub fn bytes_to_mib(bytes: u64) -> f64 {
        bytes as f64 / 1_048_576.0
    }

    /// Returns RAM total in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_total_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_total_bytes)
    }

    /// Returns RAM used in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_used_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_used_bytes)
    }

    /// Returns RAM available in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_available_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_available_bytes)
    }

    /// Returns RAM cached in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_cached_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_cached_bytes())
    }

    /// Returns Swap total in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn swap_total_gib(&self) -> f64 {
        Self::bytes_to_gib(self.swap_total_bytes)
    }

    /// Returns Swap used in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn swap_used_gib(&self) -> f64 {
        Self::bytes_to_gib(self.swap_used_bytes)
    }

    /// Returns Swap free in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn swap_free_gib(&self) -> f64 {
        Self::bytes_to_gib(self.swap_free_bytes)
    }

    /// Returns Commit total in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn commit_total_gib(&self) -> f64 {
        Self::bytes_to_gib(self.commit_total_bytes)
    }

    /// Returns Commit limit in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn commit_limit_gib(&self) -> f64 {
        Self::bytes_to_gib(self.commit_limit_bytes)
    }

    /// Returns Kernel paged pool in MiB.
    #[allow(dead_code)]
    #[inline]
    pub fn kernel_paged_mib(&self) -> f64 {
        Self::bytes_to_mib(self.kernel_paged_bytes)
    }

    /// Returns Kernel non-paged pool in MiB.
    #[allow(dead_code)]
    #[inline]
    pub fn kernel_nonpaged_mib(&self) -> f64 {
        Self::bytes_to_mib(self.kernel_nonpaged_bytes)
    }

    /// Returns total kernel pool (paged + non-paged) in MiB.
    #[allow(dead_code)]
    #[inline]
    pub fn kernel_total_mib(&self) -> f64 {
        Self::bytes_to_mib(self.kernel_paged_bytes + self.kernel_nonpaged_bytes)
    }
}

//! Live CPU frequency and brand name via Win32 APIs.
//! Primary: CallNtPowerInformation(ProcessorInformation) for CurrentMhz per core.
//! Fallback: sysinfo Cpu::frequency() (base clock only).

use std::sync::{Arc, OnceLock};
use sysinfo::System;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Power::{CallNtPowerInformation, ProcessorInformation};

#[cfg(target_os = "windows")]
const MAX_CORES: usize = 256;

/// Cached CPU brand string (read once at startup).
static CPU_BRAND: OnceLock<Arc<str>> = OnceLock::new();

/// Cached base MHz (MaxMhz from CallNtPowerInformation, read once at startup).
static BASE_MHZ: OnceLock<u64> = OnceLock::new();

pub(crate) fn decode_utf16_cstr(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end]).trim().to_string()
}

fn fallback_brand(sys: &System) -> String {
    if let Some(cpu) = sys.cpus().first() {
        let b = cpu.brand().trim();
        if !b.is_empty() {
            return b.to_string();
        }
    }
    let b = sys.global_cpu_info().brand().trim();
    if !b.is_empty() {
        return b.to_string();
    }
    if let Ok(b) = read_cpu_brand_registry() {
        let t = b.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    "x86_64 Processor".to_string()
}

/// Initializes the CPU brand and base MHz caches. Call once at startup.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn init_cpu_brand_and_base_mhz(sys: &mut System) {
    // Read base MHz via Win32 (same logic as read_cpu_frequencies but only for base)
    let n = unsafe { windows_sys::Win32::System::Threading::GetActiveProcessorCount(0) } as usize;
    let n = n.clamp(1, MAX_CORES);

    let mut buf = [ProcessorPowerInfo::default(); MAX_CORES];
    let status = unsafe {
        CallNtPowerInformation(
            ProcessorInformation,
            std::ptr::null_mut(),
            0,
            buf.as_mut_ptr() as *mut _,
            (n * std::mem::size_of::<ProcessorPowerInfo>()) as u32,
        )
    };

    let base_mhz = if status == 0 {
        let mut max_base = 0u64;
        for info in buf.iter().take(n) {
            max_base = max_base.max(info.max_mhz as u64);
        }
        max_base
    } else {
        // Fallback: sysinfo base frequency
        sys.refresh_cpu_specifics(sysinfo::CpuRefreshKind::new().with_frequency());
        sys.cpus().iter().map(|c| c.frequency()).max().unwrap_or(0)
    };
    let _ = BASE_MHZ.set(base_mhz);

    // Read CPU brand string (same logic as read_cpu_brand)
    let brand = Arc::from(fallback_brand(sys));
    let _ = CPU_BRAND.set(brand);
}

/// Initializes the CPU brand and base MHz caches (non-Windows stub).
#[cfg(not(target_os = "windows"))]
#[cfg_attr(not(target_os = "windows"), allow(dead_code, unused_mut))]
pub fn init_cpu_brand_and_base_mhz(sys: &mut System) {
    sys.refresh_cpu_specifics(sysinfo::CpuRefreshKind::new().with_frequency());
    let base = sys.cpus().iter().map(|c| c.frequency()).max().unwrap_or(0);
    let _ = BASE_MHZ.set(base);
    let brand = Arc::from(fallback_brand(sys));
    let _ = CPU_BRAND.set(brand);
}

/// Returns the cached CPU brand string. Must call `init_cpu_brand_and_base_mhz` first.
pub fn get_cpu_brand() -> Arc<str> {
    CPU_BRAND.get().cloned().unwrap_or_else(|| Arc::from("x86_64 Processor"))
}

/// Returns the cached base MHz. Must call `init_cpu_brand_and_base_mhz` first.
pub fn get_base_mhz() -> u64 {
    BASE_MHZ.get().copied().unwrap_or(0)
}

/// Per-processor power information returned by CallNtPowerInformation.
#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessorPowerInfo {
    pub number: u32,
    pub max_mhz: u32,
    pub current_mhz: u32,
    pub mhz_limit: u32,
    pub max_idle_state: u32,
    pub current_idle_state: u32,
}

/// Reads live CPU current frequency (CurrentMhz) per core.
/// Returns (base_mhz, current_mhz) where current_mhz reflects Turbo Boost / SpeedShift.
/// Base MHz is read from the cached BASE_MHZ (initialized once at startup).
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn read_cpu_frequencies() -> (u64, u64) {
    thread_local! {
        static FREQ_BUF: std::cell::RefCell<Vec<ProcessorPowerInfo>> =
            const { std::cell::RefCell::new(Vec::new()) };
    }
    let base_mhz = get_base_mhz();
    let count = (unsafe { windows_sys::Win32::System::Threading::GetActiveProcessorCount(0) }
        as usize)
        .clamp(1, MAX_CORES);
    FREQ_BUF.with(|cell| {
        let mut buf = cell.borrow_mut();
        if buf.len() != count {
            buf.resize(count, ProcessorPowerInfo::default());
        }
        let status = unsafe {
            CallNtPowerInformation(
                ProcessorInformation,
                std::ptr::null_mut(),
                0,
                buf.as_mut_ptr() as *mut _,
                (count * std::mem::size_of::<ProcessorPowerInfo>()) as u32,
            )
        };
        if status == 0 {
            let mut max_current = 0u64;
            for info in buf.iter().take(count) {
                max_current = max_current.max(info.current_mhz as u64);
            }
            (base_mhz, max_current)
        } else {
            (base_mhz, base_mhz)
        }
    })
}

/// Non-Windows stub: returns cached base MHz for both values.
#[cfg(not(target_os = "windows"))]
pub fn read_cpu_frequencies() -> (u64, u64) {
    let base_mhz = get_base_mhz();
    (base_mhz, base_mhz)
}

/// Returns the cached CPU brand string.
/// Falls back to reading from sysinfo/registry if not yet initialized (should not happen in normal operation).
pub fn read_cpu_brand(_sys: &System) -> Arc<str> {
    get_cpu_brand()
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn read_cpu_brand_registry() -> Result<String, std::io::Error> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
    };

    let subkey =
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0\0".encode_utf16().collect::<Vec<u16>>();
    let value_name = "ProcessorNameString\0".encode_utf16().collect::<Vec<u16>>();

    let mut hkey: HKEY = 0;
    let result =
        unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut hkey) };
    if result != 0 {
        return Err(std::io::Error::from_raw_os_error(result as i32));
    }

    let mut buf = [0u16; 128];
    let mut buf_size = (buf.len() * 2) as u32;
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut _,
            &mut buf_size,
        )
    };
    unsafe { RegCloseKey(hkey) };

    if result != 0 {
        return Err(std::io::Error::from_raw_os_error(result as i32));
    }
    let len = (buf_size / 2) as usize;
    Ok(decode_utf16_cstr(&buf[..len.min(buf.len())]))
}

/// Non-Windows stub: no registry on this platform.
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn read_cpu_brand_registry() -> Result<String, std::io::Error> {
    Err(std::io::Error::from_raw_os_error(1))
}

/// Two-pass CPU refresh warmup: first call primes sysinfo's internal deltas.
pub fn warmup_cpu_refresh(sys: &mut System) {
    sys.refresh_cpu_specifics(sysinfo::CpuRefreshKind::new().with_cpu_usage());
    std::thread::yield_now();
    sys.refresh_cpu_specifics(sysinfo::CpuRefreshKind::new().with_cpu_usage());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strips_nul_and_spaces() {
        let buf: Vec<u16> = "  AMD Ryzen 9 \0\0".encode_utf16().collect();
        assert_eq!(decode_utf16_cstr(&buf), "AMD Ryzen 9");
        let buf2: Vec<u16> = "abc\0def".encode_utf16().collect();
        assert_eq!(decode_utf16_cstr(&buf2), "abc");
    }
}

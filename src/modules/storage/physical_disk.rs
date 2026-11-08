//! Physical disk mapping via Win32 IOCTLs (zero admin rights).
//! Maps volume mount point to STORAGE_DEVICE_NUMBER + DISK_PERFORMANCE.

#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::IO::DeviceIoControl;

#[cfg(target_os = "windows")]
const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x002D1080;
#[cfg(target_os = "windows")]
const IOCTL_DISK_PERFORMANCE: u32 = 0x00070020;
#[cfg(target_os = "windows")]
const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x0056_0000;
#[cfg(target_os = "windows")]
const INVALID_HANDLE: HANDLE = -1isize as HANDLE;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Default)]
struct StorageDeviceNumber {
    device_type: u32,
    device_number: u32,
    partition_number: u32,
}

/// Extracts drive letter from mount point like "C:\" or "C:".
pub fn drive_letter(mount: &str) -> Option<char> {
    let bytes = mount.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        Some(bytes[0].to_ascii_uppercase() as char)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
struct VolumeHandle(HANDLE);
#[cfg(target_os = "windows")]
impl Drop for VolumeHandle {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if self.0 != 0 && self.0 != INVALID_HANDLE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn handle_cache() -> &'static Mutex<HashMap<char, isize>> {
    static CACHE: OnceLock<Mutex<HashMap<char, isize>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn cached_volume(letter: char) -> Option<isize> {
    {
        let cache = handle_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(h) = cache.get(&letter) {
            return Some(*h);
        }
    }
    let volume = open_volume(letter)?;
    let raw = volume.0;
    std::mem::forget(volume);
    let mut cache = handle_cache().lock().unwrap_or_else(|e| e.into_inner());
    if cache.len() >= 32 && !cache.contains_key(&letter) {
        for handle in cache.values() {
            if *handle != 0 && *handle != INVALID_HANDLE {
                unsafe {
                    CloseHandle(*handle as HANDLE);
                }
            }
        }
        cache.clear();
    }
    if let Some(old) = cache.insert(letter, raw) {
        if old != raw {
            unsafe {
                CloseHandle(old as HANDLE);
            }
        }
    }
    Some(raw)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn evict_handle(letter: char) {
    let mut cache = handle_cache().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(raw) = cache.remove(&letter) {
        unsafe {
            CloseHandle(raw as HANDLE);
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn open_volume(letter: char) -> Option<VolumeHandle> {
    let path = format!("\\\\.\\{}:", letter);
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            0,
        )
    };
    if handle == 0 || handle == INVALID_HANDLE {
        None
    } else {
        Some(VolumeHandle(handle))
    }
}

/// Returns (physical_disk, partition) for a mount point, or None on failure.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn physical_disk_for_mount(mount: &str) -> Option<(u32, u32)> {
    let letter = drive_letter(mount)?;
    let handle = cached_volume(letter)? as HANDLE;
    let mut out = StorageDeviceNumber::default();
    let mut returned: u32 = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_GET_DEVICE_NUMBER,
            std::ptr::null(),
            0,
            &mut out as *mut _ as *mut _,
            std::mem::size_of::<StorageDeviceNumber>() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 || returned < 12 {
        let mut buf = [0u8; 32];
        let mut returned2: u32 = 0;
        let ok2 = unsafe {
            DeviceIoControl(
                handle,
                IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
                std::ptr::null(),
                0,
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
                &mut returned2,
                std::ptr::null_mut(),
            )
        };
        if ok2 != 0 && returned2 >= 12 {
            let num_extents = u32::from_ne_bytes(buf[0..4].try_into().unwrap_or([0; 4]));
            let disk_num = u32::from_ne_bytes(buf[8..12].try_into().unwrap_or([0; 4]));
            if num_extents >= 1 {
                return Some((disk_num, 0));
            }
        }
        evict_handle(letter);
        return None;
    }
    Some((out.device_number, out.partition_number))
}

/// Returns (bytes_read, bytes_written) counters from DISK_PERFORMANCE, if any.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn perf_counters_for_mount(mount: &str) -> Option<(i64, i64)> {
    let letter = drive_letter(mount)?;
    let handle = cached_volume(letter)? as HANDLE;
    let mut buf = [0u8; 128];
    let mut returned: u32 = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_PERFORMANCE,
            std::ptr::null(),
            0,
            buf.as_mut_ptr() as *mut _,
            buf.len() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 || returned < 16 {
        evict_handle(letter);
        return None;
    }
    let read = i64::from_ne_bytes(buf[0..8].try_into().ok()?);
    let written = i64::from_ne_bytes(buf[8..16].try_into().ok()?);
    Some((read, written))
}

/// Sums hardware counters across mounts using IOCTL_DISK_PERFORMANCE.
#[allow(dead_code)]
#[cfg(target_os = "windows")]
pub fn sum_perf_counters(mounts: &[&str]) -> Option<(i64, i64)> {
    let mut read: i64 = 0;
    let mut written: i64 = 0;
    let mut any = false;
    for mount in mounts {
        if let Some((r, w)) = perf_counters_for_mount(mount) {
            read = read.saturating_add(r);
            written = written.saturating_add(w);
            any = true;
        }
    }
    any.then_some((read, written))
}

#[cfg(not(target_os = "windows"))]
pub fn physical_disk_for_mount(_mount: &str) -> Option<(u32, u32)> {
    None
}
#[cfg(not(target_os = "windows"))]
pub fn perf_counters_for_mount(_mount: &str) -> Option<(i64, i64)> {
    None
}
#[allow(dead_code)]
#[cfg(not(target_os = "windows"))]
pub fn sum_perf_counters(_mounts: &[&str]) -> Option<(i64, i64)> {
    None
}
#[allow(dead_code)]
#[cfg(not(target_os = "windows"))]
fn evict_handle(_letter: char) {}

#[cfg(test)]
#[path = "physical_disk_tests.rs"]
mod tests;

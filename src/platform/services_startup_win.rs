//! Windows-only advapi32 FFI for read-only SCM and Run-key enumeration.
use super::{
    bytes_to_words, classify_enum_step, decode_utf16_le, retry_capacity, run_key_partial,
    wide_bytes, StartupEntry, WindowsServiceEntry, ERROR_MORE_DATA,
};
use std::ptr::{null, null_mut};
type Svc = WindowsServiceEntry;
const SCM_ACCESS: u32 = 0x0005;
const SC_INFO: u32 = 0;
const SVC_WIN32: u32 = 0x30;
const SVC_ALL: u32 = 0x03;
const KEY_READ: u32 = 0x20019;
const REG_SZ: u32 = 1;
const REG_EXPAND_SZ: u32 = 2;
const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as i32 as isize;
const HKEY_LOCAL_MACHINE: isize = 0x8000_0002u32 as i32 as isize;
const MAX_SCM_PAGES: usize = 32;
const MAX_SCM_BYTES: usize = 1 << 20;
const MAX_VALUE_NAME: usize = 4096;
const MAX_VALUE_DATA: usize = 1 << 20;
const MAX_REG_RETRIES: usize = 8;

#[link(name = "advapi32")]
extern "system" {
    fn OpenSCManagerW(machine: *const u16, database: *const u16, access: u32) -> isize;
    fn EnumServicesStatusExW(
        scm: isize,
        level: u32,
        service_type: u32,
        service_state: u32,
        buffer: *mut u8,
        buffer_size: u32,
        needed: *mut u32,
        returned: *mut u32,
        resume: *mut u32,
        group: *const u16,
    ) -> i32;
    fn CloseServiceHandle(handle: isize) -> i32;
    fn RegOpenKeyExW(
        key: isize,
        sub_key: *const u16,
        options: u32,
        access: u32,
        result: *mut isize,
    ) -> i32;
    fn RegEnumValueW(
        key: isize,
        index: u32,
        name: *mut u16,
        name_size: *mut u32,
        reserved: *mut u32,
        value_type: *mut u32,
        data: *mut u8,
        data_size: *mut u32,
    ) -> i32;
    fn RegCloseKey(key: isize) -> i32;
}

#[repr(C)]
struct ServiceStatusProcess {
    service_type: u32,
    current_state: u32,
    controls_accepted: u32,
    win32_exit_code: u32,
    service_specific_exit_code: u32,
    check_point: u32,
    wait_hint: u32,
    process_id: u32,
    service_flags: u32,
}

#[repr(C)]
struct EnumServiceStatusProcessW {
    service_name: *mut u16,
    display_name: *mut u16,
    status: ServiceStatusProcess,
}

struct ScHandle(isize);
impl Drop for ScHandle {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        (self.0 != 0).then(|| unsafe { CloseServiceHandle(self.0) });
    }
}

struct RegHandle(isize);
impl Drop for RegHandle {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        (self.0 != 0).then(|| unsafe { RegCloseKey(self.0) });
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[allow(unsafe_code)]
fn call_enum(
    scm: isize,
    buffer: *mut u8,
    size: u32,
    needed: &mut u32,
    returned: &mut u32,
    resume: &mut u32,
) -> i32 {
    let f = EnumServicesStatusExW;
    unsafe { f(scm, SC_INFO, SVC_WIN32, SVC_ALL, buffer, size, needed, returned, resume, null()) }
}

/// Reads a NUL-terminated UTF-16 name from the buffer (names live AFTER the struct array).
fn read_wide(words: &[u64], cap: usize, ptr: *const u16) -> String {
    let (base, addr) = (words.as_ptr() as usize, ptr as usize);
    if ptr.is_null() || addr < base || addr >= base.saturating_add(cap) {
        return String::new();
    }
    let limit = cap.min(words.len() * 8);
    let mut pos = (addr - base) & !1;
    let mut units = Vec::new();
    while pos + 1 < limit {
        let bytes = words[pos / 8].to_le_bytes();
        let off = pos % 8;
        let unit = u16::from_le_bytes([bytes[off], bytes[off + 1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        pos += 2;
    }
    String::from_utf16_lossy(&units)
}

/// Reads each returned SCM record with `read_unaligned` and strict bounds.
#[allow(unsafe_code)]
fn parse_services(w: &[u64], structs: usize, cap: usize, count: usize, out: &mut Vec<Svc>) -> bool {
    let need = std::mem::size_of::<EnumServiceStatusProcessW>();
    let max = (structs / need).min(count);
    for i in 0..max {
        let offset = i * need;
        let ptr =
            unsafe { (w.as_ptr() as *const u8).add(offset) } as *const EnumServiceStatusProcessW;
        let row = unsafe { std::ptr::read_unaligned(ptr) };
        out.push(WindowsServiceEntry {
            name: read_wide(w, cap, row.service_name),
            display_name: read_wide(w, cap, row.display_name),
            state: row.status.current_state,
            pid: row.status.process_id,
        });
    }
    max == count
}

/// Captures every returned page before acting on `ERROR_MORE_DATA`.
#[allow(unsafe_code)]
pub(super) fn windows_services() -> (Vec<WindowsServiceEntry>, bool) {
    let scm = unsafe { OpenSCManagerW(null(), null(), SCM_ACCESS) };
    if scm == 0 {
        return (Vec::new(), true);
    }
    let _scm_guard = ScHandle(scm);

    let mut probe = [0u64; 1];
    let (mut needed, mut returned, mut resume) = (0u32, 0u32, 0u32);
    let probe_rc =
        call_enum(scm, probe.as_mut_ptr() as *mut u8, 8, &mut needed, &mut returned, &mut resume);
    let (probe_ok, _) = classify_enum_step(probe_rc);
    if needed == 0 {
        return (Vec::new(), !probe_ok);
    }
    resume = 0; // restart from the first service with the full buffer

    let record = std::mem::size_of::<EnumServiceStatusProcessW>();
    let mut words = vec![0u64; bytes_to_words(needed as usize).max(1)];
    let mut entries = Vec::new();
    let mut partial = false;
    for _page in 0..MAX_SCM_PAGES {
        let (mut need, mut got) = (0u32, 0u32);
        let ptr = words.as_mut_ptr() as *mut u8;
        let buf_bytes = words.len() * std::mem::size_of::<u64>();
        let rc = call_enum(scm, ptr, buf_bytes as u32, &mut need, &mut got, &mut resume);
        // ERROR_MORE_DATA is a BOOL failure; classify `more` before `ok`.
        let (ok, more) = classify_enum_step(rc);
        if !ok && !more {
            partial = true;
            break;
        } // real error, not truncation
        let structs = (got as usize).saturating_mul(record).min(buf_bytes);
        if !parse_services(&words, structs, buf_bytes, got as usize, &mut entries) {
            partial = true;
        }
        if ok || resume == 0 {
            break;
        }
        let cap = (need as usize).saturating_mul(MAX_SCM_PAGES).min(MAX_SCM_BYTES);
        let Some(next) = retry_capacity(ERROR_MORE_DATA, buf_bytes, need as usize, cap) else {
            partial = true;
            break;
        };
        words.resize(bytes_to_words(next).max(words.len() + 1), 0u64);
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    (entries, partial)
}

/// Retries the SAME index with larger name/data buffers on `ERROR_MORE_DATA`.
#[allow(unsafe_code)]
fn read_run_key(scope: &'static str, root: isize) -> (Vec<StartupEntry>, bool) {
    let sub_key = wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let mut key = 0isize;
    if unsafe { RegOpenKeyExW(root, sub_key.as_ptr(), 0, KEY_READ, &mut key) } != 0 {
        return (Vec::new(), true);
    }
    let _key_guard = RegHandle(key);
    let g = RegEnumValueW;
    let mut entries = Vec::new();
    let (mut index, mut partial) = (0u32, false);
    let (mut name_cap, mut data_cap) = (256usize, 8192usize);
    loop {
        let (mut name, mut data) = (Vec::new(), Vec::new());
        let (mut name_len, mut data_len, mut value_type) = (0u32, 0u32, 0u32);
        let (mut rc, mut skip) = (0i32, false);
        for _try in 0..MAX_REG_RETRIES {
            name = vec![0u16; name_cap];
            name_len = name.len() as u32;
            data = vec![0u64; bytes_to_words(data_cap)];
            data_len = (data.len() * std::mem::size_of::<u64>()) as u32;
            value_type = 0;
            let (np, dp) = (name.as_mut_ptr(), data.as_mut_ptr() as *mut u8);
            rc = unsafe {
                g(key, index, np, &mut name_len, null_mut(), &mut value_type, dp, &mut data_len)
            };
            if rc as u32 != ERROR_MORE_DATA {
                break;
            }
            let n = retry_capacity(rc as u32, name_cap, name_len as usize, MAX_VALUE_NAME);
            let d = retry_capacity(rc as u32, data_cap, data_len as usize, MAX_VALUE_DATA);
            if n.is_none() && d.is_none() {
                partial = true;
                skip = true;
                break;
            }
            name_cap = n.unwrap_or(name_cap);
            data_cap = d.unwrap_or(data_cap);
        }
        if run_key_partial(rc, skip) {
            partial = true;
        }
        if skip {
            index += 1;
            continue;
        }
        if rc != 0 {
            break;
        }
        let entry_name = String::from_utf16_lossy(&name[..(name_len as usize).min(name.len())]);
        let bytes = wide_bytes(&data);
        let take = (data_len as usize).min(bytes.len());
        let command = if matches!(value_type, REG_SZ | REG_EXPAND_SZ) {
            decode_utf16_le(&bytes[..take])
        } else {
            String::from_utf8_lossy(&bytes[..take]).into_owned()
        };
        entries.push(StartupEntry { scope, name: entry_name, command: command.trim().to_string() });
        index += 1;
    }
    (entries, partial)
}

pub(super) fn windows_startup() -> (Vec<StartupEntry>, bool) {
    let (mut entries, partial) = read_run_key("HKCU", HKEY_CURRENT_USER);
    let (hklm, hklm_partial) = read_run_key("HKLM", HKEY_LOCAL_MACHINE);
    entries.extend(hklm);
    entries.sort_by(|a, b| a.scope.cmp(b.scope).then(a.name.cmp(&b.name)));
    (entries, partial || hklm_partial)
}

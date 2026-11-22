//! Pure DXGI enumeration; system-wide adapter usage comes from PDH counters.
//! COM slots: IUnknown 0-2, EnumAdapters1 12, GetDesc1 10, QueryVideoMemoryInfo 14 (budget only).
//! dxgi1_4.h / windows-rs IDXGIAdapter3_Vtbl: [12]=RegisterHWC,
//! [13]=UnregisterHWC, [14]=QueryVideoMemoryInfo. NEVER move this call to slot 12.
use std::ffi::c_void;
#[cfg(target_os = "windows")]
#[link(name = "dxgi")]
extern "system" {
    fn CreateDXGIFactory1(riid: *const Guid, pp_factory: *mut *mut c_void) -> i32;
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}
#[cfg(target_os = "windows")]
const IID_IDXGI_FACTORY1: Guid = Guid {
    data1: 0x770a_ae78,
    data2: 0xf26f,
    data3: 0x4dba,
    data4: [0xa8, 0x29, 0x25, 0x3c, 0x83, 0xd1, 0xb3, 0x87],
};
#[cfg(target_os = "windows")]
const IID_IDXGI_ADAPTER3: Guid = Guid {
    data1: 0x6459_67a4,
    data2: 0x1392,
    data3: 0x4310,
    data4: [0xa7, 0x98, 0x80, 0x53, 0xce, 0x3e, 0x93, 0xfd],
};
#[repr(C)]
struct IUnknownVtbl {
    query_interface: unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    _add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}
#[repr(C)]
struct Factory1Vtbl {
    _base: IUnknownVtbl,
    _object: [*mut c_void; 4],
    _factory: [*mut c_void; 5],
    enum_adapters1: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> i32,
}
#[repr(C)]
struct Adapter1Vtbl {
    _base: IUnknownVtbl,
    _object: [*mut c_void; 4],
    _adapter: [*mut c_void; 3],
    get_desc1: unsafe extern "system" fn(*mut c_void, *mut DxgiAdapterDesc1) -> i32,
}
/// IDXGIAdapter3 vtable: slots 0-11 inherited, 12=RegisterHWC,
/// 13=UnregisterHWC, 14=QueryVideoMemoryInfo (per dxgi1_4.h).
#[repr(C)]
struct Adapter3Vtbl {
    _base: IUnknownVtbl,
    _object: [*mut c_void; 4],
    _adapter: [*mut c_void; 3],
    _adapter1: [*mut c_void; 1],
    _adapter2: [*mut c_void; 1],
    _teardown: [*mut c_void; 2],
    query_video_memory_info:
        unsafe extern "system" fn(*mut c_void, u32, u32, *mut DxgiQueryVideoMemoryInfo) -> i32,
}
#[repr(C)]
#[derive(Default)]
struct Luid {
    low_part: u32,
    high_part: i32,
}
#[repr(C)]
struct DxgiAdapterDesc1 {
    description: [u16; 128],
    vendor_id: u32,
    device_id: u32,
    _sub_sys_id: u32,
    _revision: u32,
    dedicated_video_memory: usize,
    _dedicated_system_memory: usize,
    shared_system_memory: usize,
    adapter_luid: Luid,
    flags: u32,
}
#[repr(C)]
#[derive(Default)]
struct DxgiQueryVideoMemoryInfo {
    budget: u64,
    current_usage: u64,
    _available_for_reservation: u64,
    _current_reservation: u64,
}
struct ComPtr {
    ptr: *mut c_void,
}
impl ComPtr {
    fn null() -> Self {
        Self { ptr: std::ptr::null_mut() }
    }
    fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
    fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
    fn ptr_mut(&mut self) -> *mut *mut c_void {
        &mut self.ptr
    }
    #[allow(unsafe_code)]
    unsafe fn vtable<T>(&self) -> &T {
        let vtbl = self.ptr as *const *const T;
        unsafe { &**vtbl }
    }
}
#[allow(unsafe_code)]
impl Drop for ComPtr {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let vtbl_ptr = self.ptr as *const *const IUnknownVtbl;
                ((**vtbl_ptr).release)(self.ptr);
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DxgiGpuAdapter {
    pub name: String,
    pub luid: u64,
    pub vendor_id: u32,
    pub device_id: u32,
    pub adapter_kind: &'static str,
    pub integrated: bool,
    pub dedicated_limit_bytes: u64,
    pub dedicated_used_bytes: u64,
    pub shared_limit_bytes: u64,
    pub shared_used_bytes: u64,
    pub dedicated_budget_bytes: u64,
    pub shared_is_spill: bool,
}
/// Combines the two halves of a DXGI LUID into one comparable value.
pub fn pack_luid(low: u32, high: i32) -> u64 {
    ((high as u64) << 32) | low as u64
}
/// True for `DXGI_ADAPTER_FLAG_SOFTWARE` or the Microsoft WARP adapter.
pub fn is_software_adapter(flags: u32, vendor_id: u32, device_id: u32) -> bool {
    (flags & 2) != 0 || (vendor_id == 0x1414 && device_id == 0x008c)
}
pub fn is_integrated_gpu(vendor_id: u32, dedicated: u64, shared: u64) -> bool {
    matches!(vendor_id, 0x8086 | 0x5143 | 0x1002)
        && dedicated <= 512 * 1024 * 1024
        && shared > dedicated
}
/// Spill is only meaningful on a discrete adapter; APUs share system RAM by design.
pub fn adapter_shared_is_spill(integrated: bool, shared_used: u64) -> bool {
    !integrated && crate::platform::pdh_gpu::is_vram_spill(shared_used)
}
/// Small carve-outs and APU vendors are shared-memory GPUs, not discrete VRAM.
pub fn classify_gpu_memory_kind(vendor: u32, ded: u64, shared: u64) -> &'static str {
    if is_integrated_gpu(vendor, ded, shared) {
        "Integrated (Shared RAM)"
    } else {
        "Discrete VRAM"
    }
}
/// Decodes a fixed UTF-16 DXGI description, trimming NULs and outer spaces.
pub fn decode_adapter_description(raw: &[u16; 128]) -> String {
    let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
    String::from_utf16_lossy(&raw[..len]).trim().to_string()
}
#[cfg(target_os = "windows")]
pub fn query_dxgi_gpus() -> Vec<DxgiGpuAdapter> {
    windows_query()
}
#[cfg(not(target_os = "windows"))]
pub fn query_dxgi_gpus() -> Vec<DxgiGpuAdapter> {
    Vec::new()
}
/// Invokes IDXGIAdapter3 slot 14 (QueryVideoMemoryInfo) on a prepared adapter3.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn query_video_memory_info(adapter3: &ComPtr) -> Option<DxgiQueryVideoMemoryInfo> {
    let vtbl = unsafe { adapter3.vtable::<Adapter3Vtbl>() };
    let mut info: DxgiQueryVideoMemoryInfo = unsafe { std::mem::zeroed() };
    if unsafe { (vtbl.query_video_memory_info)(adapter3.as_ptr(), 0, 0, &mut info) } < 0 {
        return None;
    }
    Some(info)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn query_budgets(adapter: &ComPtr, dedicated: u64, shared: u64) -> (u64, u64) {
    let mut adapter3 = ComPtr::null();
    let iunknown = unsafe { adapter.vtable::<IUnknownVtbl>() };
    let q = iunknown.query_interface;
    let hr = unsafe { q(adapter.as_ptr(), &IID_IDXGI_ADAPTER3, adapter3.ptr_mut()) };
    if hr < 0 || adapter3.is_null() {
        return (dedicated, shared);
    }
    let Some(local) = query_video_memory_info(&adapter3) else {
        return (dedicated, shared);
    };
    let budget = if local.budget > 0 { local.budget } else { dedicated };
    (budget, shared)
}
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn windows_query() -> Vec<DxgiGpuAdapter> {
    let mut factory = ComPtr::null();
    if unsafe { CreateDXGIFactory1(&IID_IDXGI_FACTORY1, factory.ptr_mut()) } < 0
        || factory.is_null()
    {
        return Vec::new();
    }
    let telemetry = crate::platform::pdh_gpu::sample_gpu_telemetry();
    let _history_ready = telemetry.history_len > 0;
    let totals = telemetry.adapters;
    let factory_vtbl = unsafe { factory.vtable::<Factory1Vtbl>() };
    let mut hardware = Vec::new();
    let mut software = Vec::new();
    for index in 0..32u32 {
        let mut adapter = ComPtr::null();
        let hr =
            unsafe { (factory_vtbl.enum_adapters1)(factory.as_ptr(), index, adapter.ptr_mut()) };
        if hr < 0 || adapter.is_null() {
            break;
        }
        let a1_vtbl = unsafe { adapter.vtable::<Adapter1Vtbl>() };
        let mut desc: DxgiAdapterDesc1 = unsafe { std::mem::zeroed() };
        if unsafe { (a1_vtbl.get_desc1)(adapter.as_ptr(), &mut desc) } < 0 {
            continue;
        }
        let dedicated = desc.dedicated_video_memory as u64;
        let shared = desc.shared_system_memory as u64;
        let luid = pack_luid(desc.adapter_luid.low_part, desc.adapter_luid.high_part);
        let (d_budget, s_limit) = query_budgets(&adapter, dedicated, shared);
        let integrated = is_integrated_gpu(desc.vendor_id, dedicated, shared);
        let used = totals.get(&luid);
        let d_used = used.map_or(0, |t| t.dedicated_used_bytes);
        let s_used = used.map_or(0, |t| t.shared_used_bytes);
        let gpu = DxgiGpuAdapter {
            name: decode_adapter_description(&desc.description),
            luid,
            vendor_id: desc.vendor_id,
            device_id: desc.device_id,
            adapter_kind: classify_gpu_memory_kind(desc.vendor_id, dedicated, shared),
            integrated,
            dedicated_limit_bytes: dedicated,
            dedicated_used_bytes: d_used,
            shared_limit_bytes: s_limit,
            shared_used_bytes: s_used,
            dedicated_budget_bytes: d_budget,
            shared_is_spill: adapter_shared_is_spill(integrated, s_used),
        };
        if is_software_adapter(desc.flags, desc.vendor_id, desc.device_id) {
            software.push(gpu);
        } else {
            hardware.push(gpu);
        }
    }
    if hardware.is_empty() {
        software
    } else {
        hardware
    }
}
#[cfg(test)]
#[path = "dxgi_gpu_tests.rs"]
mod tests;

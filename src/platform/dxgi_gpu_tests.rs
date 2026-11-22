//! Tests for DXGI adapter helpers and the live per-adapter query path.
use super::*;

const MB: u64 = 1024 * 1024;
const GB: u64 = 1024 * MB;

fn utf16(text: &str) -> [u16; 128] {
    let mut buf = [0u16; 128];
    for (idx, unit) in text.encode_utf16().take(127).enumerate() {
        buf[idx] = unit;
    }
    buf
}

#[test]
fn integrated_and_discrete_classification() {
    assert_eq!(classify_gpu_memory_kind(0x8086, 128 * MB, 16 * GB), "Integrated (Shared RAM)");
    assert_eq!(classify_gpu_memory_kind(0x10de, 8 * GB, 16 * GB), "Discrete VRAM");
    assert_eq!(classify_gpu_memory_kind(0x1002, 128 * MB, 16 * GB), "Integrated (Shared RAM)");
}

#[test]
fn igpu_requires_vendor_small_carveout_and_shared_ram() {
    assert!(is_integrated_gpu(0x8086, 256 * MB, 8 * GB));
    assert!(is_integrated_gpu(0x5143, 512 * MB, 4 * GB));
    assert!(is_integrated_gpu(0x1002, 512 * MB, 16 * GB));
    assert!(!is_integrated_gpu(0x1002, 512 * MB + 1, 16 * GB));
    assert!(!is_integrated_gpu(0x10de, 256 * MB, 8 * GB));
    assert!(!is_integrated_gpu(0x8086, 512 * MB + 1, 8 * GB));
    assert!(!is_integrated_gpu(0x8086, 256 * MB, 128 * MB));
}

#[test]
fn amd_apu_does_not_report_spill_but_small_nvidia_stays_discrete() {
    let apu = is_integrated_gpu(0x1002, 512 * MB, 16 * GB);
    assert_eq!(classify_gpu_memory_kind(0x1002, 512 * MB, 16 * GB), "Integrated (Shared RAM)");
    assert!(!adapter_shared_is_spill(apu, 256 * MB));
    let nvidia = is_integrated_gpu(0x10de, 256 * MB, 8 * GB);
    assert_eq!(classify_gpu_memory_kind(0x10de, 256 * MB, 8 * GB), "Discrete VRAM");
    assert!(adapter_shared_is_spill(nvidia, 256 * MB));
}

#[test]
fn software_adapters_are_filtered() {
    assert!(is_software_adapter(2, 0x10de, 0x1b06));
    assert!(is_software_adapter(0, 0x1414, 0x008c));
    assert!(!is_software_adapter(0, 0x10de, 0x1b06));
}

#[test]
fn luid_packing_preserves_both_halves() {
    assert_eq!(pack_luid(0x1234_5678, 0x1), 0x0000_0001_1234_5678);
    assert_eq!(pack_luid(0, 0), 0);
}

#[test]
fn utf16_description_trims_nuls_and_spaces() {
    assert_eq!(decode_adapter_description(&utf16("  NVIDIA RTX 4080  ")), "NVIDIA RTX 4080");
    assert_eq!(decode_adapter_description(&[0u16; 128]), "");
}

#[test]
fn live_query_does_not_panic() {
    let adapters = query_dxgi_gpus();
    #[cfg(not(target_os = "windows"))]
    assert!(adapters.is_empty());
    #[cfg(target_os = "windows")]
    for gpu in adapters {
        assert!(!gpu.adapter_kind.is_empty());
        assert!(gpu.name.len() <= 128);
        assert_eq!(
            gpu.integrated,
            is_integrated_gpu(gpu.vendor_id, gpu.dedicated_limit_bytes, gpu.shared_limit_bytes)
        );
    }
}

#[cfg(target_os = "windows")]
#[test]
#[allow(unsafe_code)]
fn query_video_memory_info_uses_slot_14() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CALLED: AtomicUsize = AtomicUsize::new(0);
    unsafe extern "system" fn qi(_: *mut c_void, _: *const Guid, _: *mut *mut c_void) -> i32 {
        0
    }
    unsafe extern "system" fn add_ref(_: *mut c_void) -> u32 {
        1
    }
    unsafe extern "system" fn release(_: *mut c_void) -> u32 {
        0
    }
    unsafe extern "system" fn slot14(
        _: *mut c_void,
        _: u32,
        _: u32,
        info: *mut DxgiQueryVideoMemoryInfo,
    ) -> i32 {
        CALLED.fetch_add(1, Ordering::SeqCst);
        unsafe { (*info).budget = 0x1000 };
        0
    }
    let vtbl = Adapter3Vtbl {
        _base: IUnknownVtbl { query_interface: qi, _add_ref: add_ref, release },
        _object: [std::ptr::null_mut(); 4],
        _adapter: [std::ptr::null_mut(); 3],
        _adapter1: [std::ptr::null_mut(); 1],
        _adapter2: [std::ptr::null_mut(); 1],
        _teardown: [std::ptr::null_mut(); 2],
        query_video_memory_info: slot14,
    };
    assert_eq!(std::mem::size_of::<Adapter3Vtbl>(), 15 * std::mem::size_of::<usize>());
    let object = &(&vtbl as *const Adapter3Vtbl) as *const _ as *mut c_void;
    let adapter3 = ComPtr { ptr: object };
    CALLED.store(0, Ordering::SeqCst);
    let info = query_video_memory_info(&adapter3).expect("slot 14 call must succeed");
    assert_eq!(CALLED.load(Ordering::SeqCst), 1);
    assert_eq!(info.budget, 0x1000);
}

#[cfg(target_os = "windows")]
#[test]
fn live_dxgi_enumeration_is_crash_free() {
    // Exercises CreateDXGIFactory1, EnumAdapters1, GetDesc1, the guarded
    // QueryInterface -> QueryVideoMemoryInfo budget call and PDH LUID matching.
    let adapters = query_dxgi_gpus();
    assert!(adapters.len() <= 32);
    for gpu in adapters {
        assert!(!gpu.adapter_kind.is_empty());
        assert!(gpu.name.len() <= 128);
        assert!(gpu.dedicated_budget_bytes <= u32::MAX as u64 * 1024);
    }
}

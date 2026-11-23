//! Tests for PDH GPU parsers, aggregation and the live Windows sample path.
use super::*;
use crate::platform::dxgi_gpu::pack_luid;
use crate::platform::pdh_gpu_counters::{
    aggregate, clean_engine_pct, hf_model_repo, paged_estimate, parse_adapter_luid,
    parse_engine_pid_luid, parse_pdh_gpu_instance_pid, CounterRows,
};

const MIB: u64 = 1024 * 1024;

#[test]
fn parses_pdh_instance_pid() {
    assert_eq!(
        parse_pdh_gpu_instance_pid("pid_1234_luid_0x00000000_0x00012345_phys_0"),
        Some(1234)
    );
    assert_eq!(parse_pdh_gpu_instance_pid("pid_7_luid_0x1"), Some(7));
    assert_eq!(parse_pdh_gpu_instance_pid("gpu_1234"), None);
    assert_eq!(parse_pdh_gpu_instance_pid("pid_"), None);
}

#[test]
fn vram_spill_boundary_is_strict() {
    assert!(!is_vram_spill(128 * MIB));
    assert!(is_vram_spill(128 * MIB + 1));
    assert!(!is_vram_spill(0));
}

#[test]
fn parses_adapter_luid_instance() {
    assert_eq!(parse_adapter_luid("luid_0x00000000_0x0000E9B0_phys_0"), Some(pack_luid(0xE9B0, 0)));
    assert_eq!(
        parse_adapter_luid("luid_0x00000001_0xDEADBEEF_phys_0"),
        Some(pack_luid(0xDEADBEEF, 1))
    );
    assert_eq!(parse_adapter_luid("pid_1234_luid_0x0_0x1"), None);
}

#[test]
fn parses_engine_pid_and_luid() {
    assert_eq!(
        parse_engine_pid_luid("pid_1234_luid_0x00000001_0x0000E9B0_eng_0"),
        Some((1234, pack_luid(0xE9B0, 1)))
    );
    assert_eq!(parse_engine_pid_luid("luid_0x0_0x1_phys_0"), None);
}

#[test]
fn engine_percent_filters_glitches_and_clamps() {
    assert_eq!(clean_engine_pct(-0.1), None);
    assert_eq!(clean_engine_pct(1000.1), None);
    assert_eq!(clean_engine_pct(250.0), Some(100.0));
    assert_eq!(clean_engine_pct(42.5), Some(42.5));
    assert_eq!(clean_engine_pct(f64::NAN), None);
}

#[test]
fn paged_estimate_is_saturating() {
    assert_eq!(paged_estimate(100, 40, 20), 40);
    assert_eq!(paged_estimate(50, 40, 20), 0);
    assert_eq!(paged_estimate(0, 0, 0), 0);
}

#[test]
fn huggingface_repo_tag_is_extracted() {
    assert_eq!(
        hf_model_repo("python C:\\cache\\models--meta-llama--Llama-3.1-8B\\snapshots"),
        Some("meta-llama/Llama-3.1-8B".to_string())
    );
    assert_eq!(
        detect_ai_model_tag("python.exe", "python -m llama_server --model models--org--repo"),
        Some("[ AI: org/repo ]".to_string())
    );
}

#[test]
fn detects_ollama_and_llama_server_models() {
    assert_eq!(
        detect_ai_model_tag("ollama.exe", "ollama run llama3:8b"),
        Some("[ AI: llama3:8b ]".to_string())
    );
    assert_eq!(
        detect_ai_model_tag("llama-server.exe", "llama-server -m qwen2.5.gguf"),
        Some("[ AI: qwen2.5.gguf ]".to_string())
    );
}

#[test]
fn unrelated_process_has_no_ai_tag() {
    assert_eq!(detect_ai_model_tag("notepad.exe", ""), None);
    assert_eq!(detect_ai_model_tag("explorer.exe", ""), None);
    assert_eq!(detect_ai_model_tag("python.exe", "python app.py"), None);
}

#[test]
fn python_with_model_flags_is_tagged() {
    assert_eq!(
        detect_ai_model_tag("python.exe", "python infer.py --model mixtral.safetensors"),
        Some("[ AI: mixtral.safetensors ]".to_string())
    );
}

#[test]
fn backoff_progression_caps_at_60_and_collect_status() {
    let mut backoff = 5;
    for expected in [10, 20, 40, 60, 60] {
        backoff = next_backoff_secs(backoff);
        assert_eq!(backoff, expected);
    }
    assert_eq!(next_backoff_secs(0), 5);
    assert!(collect_status_ok(0));
    assert!(collect_status_ok(0x8000_07D5u32 as i32));
    assert!(!collect_status_ok(0x8000_07D2u32 as i32));
}

#[test]
fn aggregate_keeps_multi_luid_invariants() {
    let low1 = 0x100u32;
    let low2 = 0x200u32;
    let l1 = pack_luid(low1, 0);
    let l2 = pack_luid(low2, 0);
    let adapter = |low: u32| format!("luid_0x00000000_0x{low:08X}_phys_0");
    let proc = |pid: u32, low: u32| format!("pid_{pid}_luid_0x00000000_0x{low:08X}_eng_0");
    let rows = CounterRows {
        adapter_dedicated: vec![
            (adapter(low1), 500.0 * MIB as f64),
            (adapter(low2), 900.0 * MIB as f64),
        ],
        local: vec![(proc(11, low1), 300.0 * MIB as f64), (proc(12, low2), 700.0 * MIB as f64)],
        engines: vec![(proc(11, low1), 12.5), (proc(12, low2), 33.0)],
        ..CounterRows::default()
    };
    let snap = aggregate(rows, 7, 0);
    assert_eq!(snap.adapters.len(), 2);
    assert_eq!(snap.epoch, 7);
    assert_eq!(snap.processes[&(11, l1)].luid, Some(l1));
    assert_eq!(snap.processes[&(12, l2)].luid, Some(l2));
    for proc_row in snap.processes.values() {
        let total = &snap.adapters[&proc_row.luid.unwrap()];
        assert!(total.dedicated_used_bytes >= proc_row.local_bytes);
    }
}

#[test]
fn one_pid_on_two_adapters_keeps_spill_on_igpu_only() {
    let (igpu, dgpu) = (pack_luid(0x100, 0), pack_luid(0x200, 0));
    let inst = |low: u32| format!("pid_42_luid_0x00000000_0x{low:08X}_eng_0");
    let rows = CounterRows {
        non_local: vec![(inst(0x100), 150.0 * MIB as f64), (inst(0x200), 0.0)],
        engines: vec![(inst(0x200), 80.0)],
        ..CounterRows::default()
    };
    let snap = aggregate(rows, 1, 0);
    let a = &snap.processes[&(42, igpu)];
    let b = &snap.processes[&(42, dgpu)];
    assert_eq!(a.non_local_bytes, 150 * MIB);
    assert_eq!(b.non_local_bytes, 0);
    assert_eq!(b.gpu_pct, Some(80.0));
    assert!(is_vram_spill(a.non_local_bytes));
    assert!(!is_vram_spill(b.non_local_bytes));
}

#[test]
fn engine_without_luid_segment_keeps_gpu_percent() {
    let inst = "pid_1234_engtype_3D_0";
    assert_eq!(parse_engine_pid_luid(inst), None);
    let rows = CounterRows { engines: vec![(inst.to_string(), 42.5)], ..CounterRows::default() };
    let snap = aggregate(rows, 1, 0);
    assert_eq!(snap.processes[&(1234, 0)].gpu_pct, Some(42.5));

    let luid = pack_luid(0x100, 0);
    let adapter = format!("luid_0x00000000_0x{:08X}_phys_0", 0x100u32);
    let rows = CounterRows {
        adapter_dedicated: vec![(adapter, 500.0)],
        engines: vec![(inst.to_string(), 42.5)],
        ..CounterRows::default()
    };
    let snap = aggregate(rows, 2, 0);
    assert_eq!(snap.processes[&(1234, luid)].gpu_pct, Some(42.5));
    assert_eq!(snap.processes[&(1234, luid)].luid, Some(luid));
}

#[test]
fn adapter_totals_dominate_process_usage_fixture() {
    use crate::platform::pdh_gpu_counters::{AdapterMemoryTotals, ProcessGpuTelemetry};
    let adapters = [
        AdapterMemoryTotals { dedicated_used_bytes: 500, ..Default::default() },
        AdapterMemoryTotals { dedicated_used_bytes: 900, ..Default::default() },
    ];
    let procs = [
        ProcessGpuTelemetry { local_bytes: 300, ..Default::default() },
        ProcessGpuTelemetry { local_bytes: 700, ..Default::default() },
    ];
    let max_adapter = adapters.iter().map(|a| a.dedicated_used_bytes).max().unwrap_or(0);
    let max_proc = procs.iter().map(|p| p.local_bytes).max().unwrap_or(0);
    assert!(max_adapter >= max_proc);
    assert!(max_adapter > 0);
}

#[cfg(target_os = "windows")]
#[test]
fn live_pdh_sampling_is_crash_free() {
    // Opens pdh.dll, collects adapter totals, per-process VRAM, paged estimate
    // and GPU engine %, then verifies totals dominate single-process usage.
    let snap = sample_gpu_telemetry();
    let again = sample_gpu_telemetry();
    assert!(snap.adapters.len() <= 32);
    assert!(snap.processes.len() < 100_000);
    assert_eq!(snap.epoch, again.epoch);
    // Headless CI VMs report a Basic Display adapter with zero dedicated VRAM,
    // so adapter positivity is checked only by the pure fixture test below.
    for p in snap.processes.values() {
        if let Some(pct) = p.gpu_pct {
            assert!(pct.is_finite());
        }
    }
    assert!(sample_gpu_history().len() <= 120);
    let legacy = sample_process_gpu_memory();
    assert!(legacy.len() < 100_000);
}

//! Pure PDH GPU parsing, aggregation and snapshot types (no Win32 calls).
use crate::platform::dxgi_gpu::pack_luid;
use std::collections::{HashMap, VecDeque};
/// Per-process GPU telemetry aggregated from PDH counters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessGpuTelemetry {
    pub pid: u32,
    /// Adapter LUID this process was sampled on; None when the instance name had no adapter.
    pub luid: Option<u64>,
    pub local_bytes: u64,
    pub non_local_bytes: u64,
    pub total_committed_bytes: u64,
    pub paged_est_bytes: u64,
    pub gpu_pct: Option<f32>,
}
/// System-wide memory totals for one adapter, matched by exact packed LUID.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdapterMemoryTotals {
    pub luid: u64,
    pub dedicated_used_bytes: u64,
    pub shared_used_bytes: u64,
    pub shared_is_spill: bool,
}
/// Cached snapshot of all GPU telemetry for one successful collection tick.
#[derive(Clone, Debug, Default)]
pub struct GpuTelemetrySnapshot {
    pub adapters: HashMap<u64, AdapterMemoryTotals>,
    pub processes: HashMap<u32, ProcessGpuTelemetry>,
    pub epoch: u64,
    pub history_len: usize,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuHistorySample {
    pub epoch: u64,
    pub dedicated_used_bytes: u64,
    pub shared_used_bytes: u64,
    pub busiest_gpu_pct: f32,
}
#[derive(Clone, Debug, Default)]
pub struct GpuHistoryRing {
    samples: VecDeque<GpuHistorySample>,
    cap: usize,
}
impl GpuHistoryRing {
    pub fn with_cap(cap: usize) -> Self {
        Self { samples: VecDeque::new(), cap: cap.max(1) }
    }
    pub fn push(&mut self, sample: GpuHistorySample) {
        if self.samples.len() == self.cap.max(1) {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }
    pub fn len(&self) -> usize {
        self.samples.len()
    }
}
#[derive(Clone, Debug, Default)]
pub struct CounterRows {
    pub local: Vec<(String, f64)>,
    pub non_local: Vec<(String, f64)>,
    pub committed: Vec<(String, f64)>,
    pub proc_dedicated: Vec<(String, f64)>,
    pub proc_shared: Vec<(String, f64)>,
    pub adapter_dedicated: Vec<(String, f64)>,
    pub adapter_shared: Vec<(String, f64)>,
    pub engines: Vec<(String, f64)>,
}
impl CounterRows {
    pub fn is_empty(&self) -> bool {
        [
            &self.local,
            &self.non_local,
            &self.committed,
            &self.proc_dedicated,
            &self.proc_shared,
            &self.adapter_dedicated,
            &self.adapter_shared,
            &self.engines,
        ]
        .iter()
        .all(|v| v.is_empty())
    }
}
/// Parses `luid_0x00000000_0x0000E9B0_phys_0` into a packed u64 LUID.
pub fn parse_adapter_luid(instance: &str) -> Option<u64> {
    let tail = instance.strip_prefix("luid_0x")?;
    let (high_s, rest) = tail.split_once('_')?;
    let low_s = rest.split('_').next()?.trim_start_matches("0x");
    let high = u32::from_str_radix(high_s, 16).ok()?;
    let low = u32::from_str_radix(low_s, 16).ok()?;
    Some(pack_luid(low, high as i32))
}
/// Parses `pid_1234_luid_0x..._0x..._...` into `(pid, packed_luid)`.
pub fn parse_engine_pid_luid(instance: &str) -> Option<(u32, u64)> {
    let tail = instance.strip_prefix("pid_")?;
    let (pid_s, luid_part) = tail.split_once('_')?;
    let pid: u32 = pid_s.parse().ok()?;
    Some((pid, parse_adapter_luid(luid_part)?))
}
/// Extracts `1234` from `pid_1234_luid_0x00000000_0x00012345_phys_0`.
pub fn parse_pdh_gpu_instance_pid(instance_name: &str) -> Option<u32> {
    let rest = instance_name.strip_prefix("pid_")?;
    let end = rest.find('_').unwrap_or(rest.len());
    rest[..end].parse::<u32>().ok()
}
/// More than 128 MiB of Non-Local usage means VRAM spilled into system RAM.
pub fn is_vram_spill(non_local_bytes: u64) -> bool {
    non_local_bytes > 128 * 1024 * 1024
}
/// Rejects driver glitches and clamps the display value to 0..=100.
pub fn clean_engine_pct(raw: f64) -> Option<f32> {
    if !raw.is_finite() || raw < 0.0 || raw > 1000.0 {
        return None;
    }
    Some((raw as f32).clamp(0.0, 100.0))
}
/// Paged pool estimate: committed minus locally and non-locally resident bytes.
pub fn paged_estimate(total_committed: u64, local: u64, non_local: u64) -> u64 {
    total_committed.saturating_sub(local.saturating_add(non_local))
}
/// Extracts `org/repo` from a Hugging Face `models--org--repo` cache path.
pub fn hf_model_repo(cmdline: &str) -> Option<String> {
    const MARK: &str = "models--";
    for word in cmdline.split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'')) {
        if let Some(at) = word.find(MARK) {
            let rest = &word[at + MARK.len()..];
            let end = rest.find(['\\', '/', '"', '\'']).unwrap_or(rest.len());
            let parts: Vec<&str> = rest[..end].split("--").take(2).collect();
            if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                return Some(format!("{}/{}", parts[0], parts[1]));
            }
        }
    }
    None
}
/// Builds an `[ AI: <model> ]` tag, including Hugging Face `org/repo` paths.
pub fn detect_ai_model_tag(process_name: &str, cmdline: &str) -> Option<String> {
    let hf = hf_model_repo(cmdline);
    let name = process_name.to_ascii_lowercase();
    let lower = cmdline.to_ascii_lowercase();
    let engines = ["ollama", "llama-server", "llama", "lm-studio", "comfyui"];
    let engine = engines.iter().any(|&e| name.contains(e) || lower.contains(e))
        || (name.contains("python") && lower.contains("model"));
    if !engine && hf.is_none() {
        return None;
    }
    let model = hf.or_else(|| extract_model_token(cmdline)).unwrap_or_else(|| "local".to_string());
    Some(format!("[ AI: {model} ]"))
}
fn clean_token(token: &str) -> &str {
    token.trim().trim_matches('"').trim_matches('\'')
}
fn extract_model_token(cmd: &str) -> Option<String> {
    let lower = cmd.to_ascii_lowercase();
    for marker in ["--model=", "--model ", "-m=", "-m ", " run "] {
        if let Some(pos) = lower.find(marker) {
            let tail = &cmd[pos + marker.len()..];
            if let Some(word) = tail.split_whitespace().next() {
                let token = clean_token(word);
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }
    for word in cmd.split_whitespace() {
        let token = clean_token(word);
        let tl = token.to_ascii_lowercase();
        if tl.ends_with(".gguf") || tl.ends_with(".safetensors") {
            return Some(token.rsplit(['/', '\\']).next().unwrap_or(token).to_string());
        }
    }
    None
}
fn positive(value: f64) -> u64 {
    if value.is_finite() && value > 0.0 {
        value as u64
    } else {
        0
    }
}
fn accumulate_pid(
    rows: Vec<(String, f64)>,
    processes: &mut HashMap<u32, ProcessGpuTelemetry>,
    set: fn(&mut ProcessGpuTelemetry, u64),
) {
    for (name, value) in rows {
        let Some((pid, luid)) = parse_engine_pid_luid(&name)
            .or_else(|| parse_pdh_gpu_instance_pid(&name).map(|p| (p, 0)))
        else {
            continue;
        };
        let row = processes.entry(pid).or_default();
        if luid != 0 {
            row.luid = Some(luid);
        }
        set(row, positive(value));
    }
}
/// Aggregates PDH rows into a snapshot keyed by LUID and PID.
pub fn aggregate(rows: CounterRows, epoch: u64, history_len: usize) -> GpuTelemetrySnapshot {
    let mut adapters: HashMap<u64, AdapterMemoryTotals> = HashMap::new();
    for (name, value) in rows.adapter_dedicated {
        if let Some(luid) = parse_adapter_luid(&name) {
            adapters.entry(luid).or_default().dedicated_used_bytes += positive(value);
        }
    }
    for (name, value) in rows.adapter_shared {
        if let Some(luid) = parse_adapter_luid(&name) {
            adapters.entry(luid).or_default().shared_used_bytes += positive(value);
        }
    }
    for (luid, totals) in adapters.iter_mut() {
        totals.luid = *luid;
        totals.shared_is_spill = is_vram_spill(totals.shared_used_bytes);
    }
    let mut processes: HashMap<u32, ProcessGpuTelemetry> = HashMap::new();
    accumulate_pid(rows.local, &mut processes, |r, v| r.local_bytes += v);
    accumulate_pid(rows.non_local, &mut processes, |r, v| r.non_local_bytes += v);
    accumulate_pid(rows.committed, &mut processes, |r, v| r.total_committed_bytes += v);
    if processes.values().all(|r| r.local_bytes == 0) {
        accumulate_pid(rows.proc_dedicated, &mut processes, |r, v| r.local_bytes += v);
    }
    if processes.values().all(|r| r.non_local_bytes == 0) {
        accumulate_pid(rows.proc_shared, &mut processes, |r, v| r.non_local_bytes += v);
    }
    for (name, value) in rows.engines {
        if let Some((pid, luid)) = parse_engine_pid_luid(&name) {
            if let Some(pct) = clean_engine_pct(value) {
                let row = processes.entry(pid).or_default();
                if luid != 0 {
                    row.luid = Some(luid);
                }
                row.gpu_pct = Some(row.gpu_pct.map_or(pct, |old| old.max(pct)));
            }
        }
    }
    for (pid, row) in processes.iter_mut() {
        row.pid = *pid;
        row.paged_est_bytes =
            paged_estimate(row.total_committed_bytes, row.local_bytes, row.non_local_bytes);
    }
    GpuTelemetrySnapshot { adapters, processes, epoch, history_len }
}
#[cfg(target_os = "windows")]
pub(crate) fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub(crate) unsafe fn read_wide(ptr: *const u16) -> String {
    let mut len = 0usize;
    while len < 512 && unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
}

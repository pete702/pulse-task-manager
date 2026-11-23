//! Pure PDH GPU parsing, aggregation and snapshot types (no Win32 calls).
#[cfg(any(target_os = "windows", test))]
use crate::platform::dxgi_gpu::pack_luid;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::collections::VecDeque;
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
    /// False means the PDH counter was unavailable, not that the value was zero.
    pub local_valid: bool,
    pub non_local_valid: bool,
}
/// System-wide memory totals for one adapter, matched by exact packed LUID.
#[cfg(any(target_os = "windows", test))]
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
    #[cfg(any(target_os = "windows", test))]
    pub adapters: HashMap<u64, AdapterMemoryTotals>,
    pub processes: HashMap<(u32, u64), ProcessGpuTelemetry>,
    pub epoch: u64,
}
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg(target_os = "windows")]
pub struct GpuHistorySample {
    pub epoch: u64,
    pub dedicated_used_bytes: u64,
    pub shared_used_bytes: u64,
    pub busiest_gpu_pct: f32,
}
#[derive(Clone, Debug, Default)]
#[cfg(target_os = "windows")]
pub struct GpuHistoryRing {
    samples: VecDeque<GpuHistorySample>,
    cap: usize,
}
#[cfg(target_os = "windows")]
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
#[cfg(any(target_os = "windows", test))]
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
#[cfg(target_os = "windows")]
impl CounterRows {
    pub fn is_empty(&self) -> bool {
        self.local.is_empty()
            && self.non_local.is_empty()
            && self.committed.is_empty()
            && self.proc_dedicated.is_empty()
            && self.proc_shared.is_empty()
            && self.adapter_dedicated.is_empty()
            && self.adapter_shared.is_empty()
            && self.engines.is_empty()
    }
}
/// Parses `luid_0x00000000_0x0000E9B0_phys_0` into a packed u64 LUID.
#[cfg(any(target_os = "windows", test))]
pub fn parse_adapter_luid(instance: &str) -> Option<u64> {
    let tail = instance.strip_prefix("luid_0x")?;
    let (high_s, rest) = tail.split_once('_')?;
    let low_s = rest.split('_').next()?.trim_start_matches("0x");
    let (high, low) = (u32::from_str_radix(high_s, 16).ok()?, u32::from_str_radix(low_s, 16).ok()?);
    Some(pack_luid(low, high as i32))
}
/// Parses `pid_1234_luid_0x..._0x..._...` into `(pid, packed_luid)`.
#[cfg(any(target_os = "windows", test))]
pub fn parse_engine_pid_luid(instance: &str) -> Option<(u32, u64)> {
    let tail = instance.strip_prefix("pid_")?;
    let (pid_s, luid_part) = tail.split_once('_')?;
    let pid: u32 = pid_s.parse().ok()?;
    Some((pid, parse_adapter_luid(luid_part)?))
}
#[cfg(any(target_os = "windows", test))]
pub fn parse_pdh_gpu_instance_pid(instance_name: &str) -> Option<u32> {
    let rest = instance_name.strip_prefix("pid_")?;
    rest[..rest.find('_').unwrap_or(rest.len())].parse::<u32>().ok()
}
/// More than 128 MiB of Non-Local usage means VRAM spilled into system RAM.
pub fn is_vram_spill(non_local_bytes: u64) -> bool {
    non_local_bytes > 128 * 1024 * 1024
}
/// Rejects driver glitches and clamps the display value to 0..=100.
#[cfg(any(target_os = "windows", test))]
pub fn clean_engine_pct(raw: f64) -> Option<f32> {
    (raw.is_finite() && (0.0..=1000.0).contains(&raw)).then_some((raw as f32).clamp(0.0, 100.0))
}
/// Paged pool estimate: committed minus locally and non-locally resident bytes.
#[cfg(any(target_os = "windows", test))]
pub fn paged_estimate(total_committed: u64, local: u64, non_local: u64) -> u64 {
    total_committed.saturating_sub(local.saturating_add(non_local))
}
/// Extracts `org/repo` from a Hugging Face `models--org--repo` cache path.
pub fn hf_model_repo(cmdline: &str) -> Option<String> {
    for word in cmdline.split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'')) {
        if let Some(at) = word.find("models--") {
            let rest = &word[at + 8..];
            let end = rest.find(['\\', '/', '"', '\'']).unwrap_or(rest.len());
            let mut it = rest[..end].split("--");
            let (org, repo) = (it.next()?, it.next()?);
            if !org.is_empty() && !repo.is_empty() {
                return Some(format!("{org}/{repo}"));
            }
        }
    }
    None
}
/// Builds an `[ AI: <model> ]` tag, including Hugging Face `org/repo` paths.
pub fn detect_ai_model_tag(process_name: &str, cmdline: &str) -> Option<String> {
    let hf = hf_model_repo(cmdline);
    let (name, lower) = (process_name.to_ascii_lowercase(), cmdline.to_ascii_lowercase());
    let engines = ["ollama", "llama-server", "llama", "lm-studio", "comfyui"];
    let engine = engines.iter().any(|&e| name.contains(e) || lower.contains(e))
        || (name.contains("python") && lower.contains("model"));
    if !engine && hf.is_none() {
        return None;
    }
    let model = hf.or_else(|| extract_model_token(cmdline)).unwrap_or_else(|| "local".to_string());
    Some(format!("[ AI: {model} ]"))
}
fn extract_model_token(cmd: &str) -> Option<String> {
    let lower = cmd.to_ascii_lowercase();
    for marker in ["--model=", "--model ", "-m=", "-m ", " run "] {
        if let Some(pos) = lower.find(marker) {
            if let Some(word) = cmd[pos + marker.len()..].split_whitespace().next() {
                let token = word.trim().trim_matches('"').trim_matches('\'');
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }
    cmd.split_whitespace()
        .map(|w| w.trim().trim_matches('"').trim_matches('\''))
        .find(|t| {
            t.to_ascii_lowercase().ends_with(".gguf")
                || t.to_ascii_lowercase().ends_with(".safetensors")
        })
        .map(|t| t.rsplit(['/', '\\']).next().unwrap_or(t).to_string())
}
#[cfg(any(target_os = "windows", test))]
fn positive(value: f64) -> u64 {
    if value.is_finite() && value > 0.0 {
        value as u64
    } else {
        0
    }
}
#[cfg(any(target_os = "windows", test))]
fn pid_key(name: &str) -> Option<(u32, u64)> {
    parse_engine_pid_luid(name).or_else(|| parse_pdh_gpu_instance_pid(name).map(|p| (p, 0)))
}
#[cfg(any(target_os = "windows", test))]
fn pid_amounts(rows: Vec<(String, f64)>) -> HashMap<(u32, u64), u64> {
    rows.into_iter().filter_map(|(n, v)| pid_key(&n).map(|k| (k, positive(v)))).collect()
}
#[cfg(any(target_os = "windows", test))]
fn accumulate_pid<F: Fn(&mut ProcessGpuTelemetry, u64)>(
    rows: Vec<(String, f64)>,
    processes: &mut HashMap<(u32, u64), ProcessGpuTelemetry>,
    set: F,
) {
    for (name, value) in rows {
        if let Some(key) = pid_key(&name) {
            let row = processes.entry(key).or_default();
            set(row, positive(value));
        }
    }
}
/// Aggregates PDH rows into a snapshot keyed by (PID, LUID) and by LUID.
#[cfg(any(target_os = "windows", test))]
pub fn aggregate(rows: CounterRows, epoch: u64, _history_len: usize) -> GpuTelemetrySnapshot {
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
    let mut processes: HashMap<(u32, u64), ProcessGpuTelemetry> = HashMap::new();
    accumulate_pid(rows.local, &mut processes, |r, v| {
        r.local_bytes += v;
        r.local_valid = true;
    });
    accumulate_pid(rows.non_local, &mut processes, |r, v| {
        r.non_local_bytes += v;
        r.non_local_valid = true;
    });
    accumulate_pid(rows.committed, &mut processes, |r, v| r.total_committed_bytes += v);
    let dedicated = pid_amounts(rows.proc_dedicated);
    let shared = pid_amounts(rows.proc_shared);
    for (key, v) in dedicated {
        let row = processes.entry(key).or_default();
        if !row.local_valid {
            row.local_bytes += v;
            row.local_valid = true;
        }
    }
    for (key, v) in shared {
        let row = processes.entry(key).or_default();
        if !row.non_local_valid {
            row.non_local_bytes += v;
            row.non_local_valid = true;
        }
    }
    let solo_luid = if adapters.len() == 1 { *adapters.keys().next().unwrap_or(&0) } else { 0 };
    for (name, value) in rows.engines {
        if let Some((pid, raw_luid)) = pid_key(&name) {
            let luid = if raw_luid == 0 { solo_luid } else { raw_luid };
            if let Some(pct) = clean_engine_pct(value) {
                let row = processes.entry((pid, luid)).or_default();
                row.gpu_pct = Some(row.gpu_pct.map_or(pct, |old| old.max(pct)));
            }
        }
    }
    for ((pid, luid), row) in processes.iter_mut() {
        row.pid = *pid;
        row.luid = (*luid != 0).then_some(*luid);
        row.paged_est_bytes =
            paged_estimate(row.total_committed_bytes, row.local_bytes, row.non_local_bytes);
    }
    GpuTelemetrySnapshot { adapters, processes, epoch }
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

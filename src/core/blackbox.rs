//! 60-second in-RAM black box: samples stutter signals and records freeze verdicts.
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use super::SystemSnapshot;
use crate::util::format_timestamp_now;

pub const BLACKBOX_CAPACITY: usize = 60;
const DISK_MB_S_THRESHOLD: f32 = 350.0;
const COMMIT_PCT_THRESHOLD: f32 = 88.0;
const PEAK_CORE_PCT_THRESHOLD: f32 = 92.0;
const JITTER_MS_THRESHOLD: f32 = 25.0;
const PACKET_LOSS_PCT_THRESHOLD: f32 = 5.0;

#[derive(Clone, Debug, PartialEq)]
pub struct BlackboxSample {
    /// Monotonic id used to expire a verdict when its sample leaves the ring.
    pub seq: u64,
    pub timestamp: String,
    pub cpu_pct: f32,
    pub peak_core_pct: f32,
    pub ram_pct: f32,
    pub commit_pct: f32,
    pub disk_mb_s: f32,
    pub ping_ms: f32,
    pub jitter_ms: f32,
    pub culprit_name: String,
    pub verdict: Option<String>,
}

/// Pure inputs for one stutter verdict evaluation.
pub struct VerdictInput<'a> {
    pub time: &'a str,
    pub peak_core_pct: f32,
    pub commit_pct: f32,
    pub disk_mb_s: f32,
    pub jitter_ms: f32,
    pub packet_loss_pct: f32,
    pub disk_spike: bool,
    pub mem_pressure: bool,
    pub cpu_culprit: &'a str,
    pub disk_culprit: &'a str,
    pub mem_culprit: &'a str,
}

fn culprit_or_unknown(name: &str) -> &str {
    if name.trim().is_empty() {
        "unknown"
    } else {
        name
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictKind {
    Disk,
    Memory,
    Cpu,
    Network,
}

/// Resolves the winning signal in strict precedence order.
pub fn verdict_kind(input: &VerdictInput<'_>) -> Option<VerdictKind> {
    if input.disk_mb_s >= DISK_MB_S_THRESHOLD || input.disk_spike {
        Some(VerdictKind::Disk)
    } else if input.commit_pct > COMMIT_PCT_THRESHOLD && input.mem_pressure {
        Some(VerdictKind::Memory)
    } else if input.peak_core_pct > PEAK_CORE_PCT_THRESHOLD {
        Some(VerdictKind::Cpu)
    } else if input.jitter_ms > JITTER_MS_THRESHOLD
        || input.packet_loss_pct > PACKET_LOSS_PCT_THRESHOLD
    {
        Some(VerdictKind::Network)
    } else {
        None
    }
}

/// The culprit must come from the same predicate that fired the verdict.
pub fn culprit_for_verdict(kind: Option<VerdictKind>, disk: &str, mem: &str, cpu: &str) -> String {
    match kind {
        Some(VerdictKind::Disk) => culprit_or_unknown(disk).to_string(),
        Some(VerdictKind::Memory) => culprit_or_unknown(mem).to_string(),
        Some(VerdictKind::Cpu) => culprit_or_unknown(cpu).to_string(),
        Some(VerdictKind::Network) | None => String::new(),
    }
}

/// Evaluates the four freeze signals in precedence order; None means nominal.
pub fn evaluate_stutter_verdict(input: &VerdictInput<'_>) -> Option<String> {
    match verdict_kind(input)? {
        VerdictKind::Disk => Some(format!(
            "{} | Freeze risk: {} (Disk {:.0} MB/s)",
            input.time,
            culprit_or_unknown(input.disk_culprit),
            input.disk_mb_s
        )),
        VerdictKind::Memory => Some(format!(
            "{} | Memory thrash: {} (Commit {:.0}%)",
            input.time,
            culprit_or_unknown(input.mem_culprit),
            input.commit_pct
        )),
        VerdictKind::Cpu => Some(format!(
            "{} | 1-Core {:.0}%: {}",
            input.time,
            input.peak_core_pct,
            culprit_or_unknown(input.cpu_culprit)
        )),
        VerdictKind::Network => {
            Some(format!("{} | Net spike: jitter {:.0} ms", input.time, input.jitter_ms))
        }
    }
}

#[derive(Default)]
struct Blackbox {
    samples: VecDeque<BlackboxSample>,
    last_verdict: Option<(u64, String)>,
    next_seq: u64,
    last_key: u64,
}

static BLACKBOX: OnceLock<Mutex<Blackbox>> = OnceLock::new();

fn state() -> &'static Mutex<Blackbox> {
    BLACKBOX.get_or_init(|| Mutex::new(Blackbox::default()))
}

fn key_for(snap: &SystemSnapshot) -> u64 {
    if let Some(mem) = &snap.memory {
        if mem.snapshot_id != 0 {
            return mem.snapshot_id;
        }
    }
    let total = snap.cpu.as_ref().map(|c| c.total_pct).unwrap_or(0.0);
    snap.uptime_secs.wrapping_mul(1000).wrapping_add((total * 10.0) as u64)
}

fn top_cpu_culprit(snap: &SystemSnapshot) -> String {
    let Some(cpu) = &snap.cpu else { return String::new() };
    cpu.raw_processes
        .iter()
        .max_by(|a, b| a.raw_cpu_usage.total_cmp(&b.raw_cpu_usage))
        .map(|p| p.name.to_string())
        .unwrap_or_default()
}

fn top_disk_culprit(snap: &SystemSnapshot) -> String {
    if let Some(spike) = &snap.storage.last_io_spike {
        return spike.process_name.to_string();
    }
    if let Some((_pid, name, _mb)) = &snap.storage.top_io_process {
        return name.to_string();
    }
    let Some(cpu) = &snap.cpu else { return String::new() };
    cpu.raw_processes
        .iter()
        .max_by(|a, b| {
            (a.disk_read_mb_s + a.disk_write_mb_s)
                .total_cmp(&(b.disk_read_mb_s + b.disk_write_mb_s))
        })
        .map(|p| p.name.to_string())
        .unwrap_or_default()
}

fn sample_from(snap: &SystemSnapshot) -> BlackboxSample {
    let timestamp = format_timestamp_now();
    let cpu_pct = snap.cpu.as_ref().map(|c| c.total_pct).unwrap_or(0.0);
    let peak_core_pct = snap.cpu.as_ref().map(|c| c.hottest_core_pct).unwrap_or(0.0);
    let ram_pct = snap.memory.as_ref().map(|m| m.ram_usage_percent()).unwrap_or(0.0);
    let commit_pct = snap.memory.as_ref().map(|m| m.commit_usage_percent()).unwrap_or(0.0);
    let disk_mb_s = snap.storage.read_mb_s + snap.storage.write_mb_s;
    let ping_ms = snap.network.ping_ms.unwrap_or(0.0);
    let jitter_ms = snap.network.jitter_ms.unwrap_or(0.0);
    let cpu_culprit = top_cpu_culprit(snap);
    let disk_culprit = top_disk_culprit(snap);
    let mem_culprit = snap
        .memory
        .as_ref()
        .and_then(|m| m.latest_spike.as_ref())
        .map(|s| s.culprit_name.to_string())
        .unwrap_or_default();
    let mem_pressure =
        snap.memory.as_ref().is_some_and(|m| m.latest_spike.is_some() || m.has_heavy_process);
    let input = VerdictInput {
        time: &timestamp,
        peak_core_pct,
        commit_pct,
        disk_mb_s,
        jitter_ms,
        packet_loss_pct: snap.network.packet_loss_pct,
        disk_spike: snap.storage.last_io_spike.is_some(),
        mem_pressure,
        cpu_culprit: &cpu_culprit,
        disk_culprit: &disk_culprit,
        mem_culprit: &mem_culprit,
    };
    let kind = verdict_kind(&input);
    let verdict = evaluate_stutter_verdict(&input);
    let culprit_name = culprit_for_verdict(kind, &disk_culprit, &mem_culprit, &cpu_culprit);
    BlackboxSample {
        seq: 0,
        timestamp,
        cpu_pct,
        peak_core_pct,
        ram_pct,
        commit_pct,
        disk_mb_s,
        ping_ms,
        jitter_ms,
        culprit_name,
        verdict,
    }
}

fn push_sample(mut sample: BlackboxSample) {
    let mut bb = state().lock().unwrap_or_else(|e| e.into_inner());
    sample.seq = bb.next_seq;
    bb.next_seq = bb.next_seq.wrapping_add(1);
    if let Some(verdict) = &sample.verdict {
        bb.last_verdict = Some((sample.seq, verdict.clone()));
    }
    bb.samples.push_back(sample);
    while bb.samples.len() > BLACKBOX_CAPACITY {
        bb.samples.pop_front();
    }
}

/// Records one new telemetry snapshot (deduplicated by snapshot identity).
pub fn observe(snap: Option<&SystemSnapshot>) {
    let Some(snap) = snap else { return };
    let key = key_for(snap);
    {
        let bb = state().lock().unwrap_or_else(|e| e.into_inner());
        if bb.last_key == key {
            return;
        }
    }
    state().lock().unwrap_or_else(|e| e.into_inner()).last_key = key;
    push_sample(sample_from(snap));
}

/// Most recent freeze verdict, only while its originating sample is in the 60 s ring.
pub fn latest_verdict() -> Option<String> {
    let mut bb = state().lock().unwrap_or_else(|e| e.into_inner());
    let (seq, verdict) = bb.last_verdict.clone()?;
    if !bb.samples.iter().any(|s| s.seq == seq && s.verdict.is_some()) {
        bb.last_verdict = None;
        return None;
    }
    Some(verdict)
}

/// Clone of the current 60-second window for export.
pub fn snapshot_samples() -> Vec<BlackboxSample> {
    state().lock().unwrap_or_else(|e| e.into_inner()).samples.iter().cloned().collect()
}

#[cfg(test)]
pub fn sample_count() -> usize {
    state().lock().unwrap_or_else(|e| e.into_inner()).samples.len()
}

/// Writes the last minute of blackbox data to the config/portable directory.
pub fn export_lag_receipt() -> Result<PathBuf, String> {
    super::blackbox_export::export_lag_receipt()
}

#[cfg(test)]
#[path = "blackbox_tests.rs"]
mod tests;

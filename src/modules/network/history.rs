//! Network history: separated traffic and latency series plus per-adapter.
//! Traffic always records RX/TX. Latency only records when the probe
//! succeeded (None ping inserts a gap, never a fake 0.0).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::core::downsampling::lttb_downsample;
use crate::core::ring_history::{is_newer, window_bounds, HISTORY_CAPACITY};

type Sample = (f64, f32, f32, Option<f32>, Option<f32>);
type AdapterSample = (f64, f32, f32);
type SeriesCache = (Option<String>, usize, u64, Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>);
type LatencyCache = (usize, u64, Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>);

fn push_gap_anchored(pts: &mut Vec<[f64; 2]>, x: f64, y: f64) {
    if let Some(&[px, _]) = pts.last() {
        if x - px > 4.5 / 60.0 {
            pts.extend([[px + 0.01, 0.0], [x - 0.01, 0.0]]);
        }
    }
    pts.push([x, y]);
}

#[derive(Clone, Debug, Default)]
pub struct NetworkHistory {
    samples: VecDeque<Sample>,
    per_adapter: HashMap<String, VecDeque<AdapterSample>>,
    per_adapter_order: VecDeque<String>,
    revision: u64,
    cached_traffic: Option<SeriesCache>,
    cached_latency: Option<LatencyCache>,
}

impl NetworkHistory {
    pub fn new() -> Self {
        Self::default()
    }
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
    pub fn push(&mut self, t: f64, rx: f32, tx: f32, ping: Option<f32>, jitter: Option<f32>) {
        if !is_newer(self.samples.back().map(|s| s.0), t) {
            return;
        }
        if self.samples.len() == HISTORY_CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back((t, rx, tx, ping, jitter));
        self.revision = self.revision.wrapping_add(1);
        self.cached_traffic = None;
        self.cached_latency = None;
    }
    pub fn push_adapter(&mut self, name: &str, t: f64, rx: f32, tx: f32) {
        if !self.per_adapter.contains_key(name) {
            crate::util::insert_capped_fifo(
                &mut self.per_adapter,
                &mut self.per_adapter_order,
                32,
                name.to_owned(),
                VecDeque::new(),
            );
        }
        let Some(entry) = self.per_adapter.get_mut(name) else { return };
        if !is_newer(entry.back().map(|s| s.0), t) {
            return;
        }
        if entry.len() == HISTORY_CAPACITY {
            entry.pop_front();
        }
        entry.push_back((t, rx, tx));
        self.revision = self.revision.wrapping_add(1);
        self.cached_traffic = None;
    }
    #[allow(clippy::type_complexity)]
    pub fn traffic_series_scoped(
        &mut self,
        scope: Option<&str>,
        window_secs: usize,
    ) -> (Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>) {
        let key: Option<String> = scope.map(str::to_owned);
        if let Some((s, win, rev, a, b)) = self.cached_traffic.clone() {
            if rev == self.revision && win == window_secs && s == key {
                return (a, b);
            }
        }
        let (a, b) = match scope {
            Some(name) => self.build_adapter(name, window_secs),
            None => self.build_total(window_secs),
        };
        let out = (Arc::new(a), Arc::new(b));
        self.cached_traffic =
            Some((key, window_secs, self.revision, Arc::clone(&out.0), Arc::clone(&out.1)));
        out
    }
    #[allow(clippy::type_complexity)]
    pub fn latency_series(
        &mut self,
        window_secs: usize,
    ) -> (Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>) {
        if let Some((win, rev, a, b)) = self.cached_latency.clone() {
            if rev == self.revision && win == window_secs {
                return (a, b);
            }
        }
        let (a, b) = self.build_latency(window_secs);
        let out = (Arc::new(a), Arc::new(b));
        self.cached_latency =
            Some((window_secs, self.revision, Arc::clone(&out.0), Arc::clone(&out.1)));
        out
    }
    fn build_total(&self, window_secs: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        if self.samples.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let latest = self.samples.back().map(|s| s.0).unwrap_or(0.0);
        let win = window_secs as f64;
        let window_min = win / 60.0;
        let timestamps: VecDeque<f64> = self.samples.iter().map(|s| s.0).collect();
        let (start_idx, _) = window_bounds(self.samples.len(), latest, &timestamps, win);
        let mut first = Vec::new();
        let mut second = Vec::new();
        for s in self.samples.iter().skip(start_idx) {
            let x = -(latest - s.0) / 60.0;
            push_gap_anchored(&mut first, x, s.1 as f64);
            push_gap_anchored(&mut second, x, s.2 as f64);
        }
        if start_idx > 0 && !first.is_empty() && first[0][0] > -window_min {
            let prev = &self.samples[start_idx - 1];
            first.insert(0, [-window_min, prev.1 as f64]);
            second.insert(0, [-window_min, prev.2 as f64]);
        } else if first.len() == 1 {
            let ay = first[0][1];
            let by = second[0][1];
            first.insert(0, [-0.001, ay]);
            second.insert(0, [-0.001, by]);
        }
        slim(first, second)
    }
    fn build_adapter(&self, scope: &str, window_secs: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        let Some(series) = self.per_adapter.get(scope) else { return (Vec::new(), Vec::new()) };
        if series.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let latest = series.back().map(|s| s.0).unwrap_or(0.0);
        let win = window_secs as f64;
        let window_min = win / 60.0;
        let timestamps: VecDeque<f64> = series.iter().map(|s| s.0).collect();
        let (start_idx, _) = window_bounds(series.len(), latest, &timestamps, win);
        let mut first = Vec::new();
        let mut second = Vec::new();
        for s in series.iter().skip(start_idx) {
            let x = -(latest - s.0) / 60.0;
            push_gap_anchored(&mut first, x, s.1 as f64);
            push_gap_anchored(&mut second, x, s.2 as f64);
        }
        if start_idx > 0 && !first.is_empty() && first[0][0] > -window_min {
            let prev = &series[start_idx - 1];
            first.insert(0, [-window_min, prev.1 as f64]);
            second.insert(0, [-window_min, prev.2 as f64]);
        } else if first.len() == 1 {
            let ay = first[0][1];
            let by = second[0][1];
            first.insert(0, [-0.001, ay]);
            second.insert(0, [-0.001, by]);
        }
        slim(first, second)
    }
    fn build_latency(&self, window_secs: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        if self.samples.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let latest = self.samples.back().map(|s| s.0).unwrap_or(0.0);
        let win = window_secs as f64;
        let window_min = win / 60.0;
        let timestamps: VecDeque<f64> = self.samples.iter().map(|s| s.0).collect();
        let (start_idx, _) = window_bounds(self.samples.len(), latest, &timestamps, win);
        let mut first = Vec::new();
        let mut second = Vec::new();
        for s in self.samples.iter().skip(start_idx) {
            let x = -(latest - s.0) / 60.0;
            push_gap_anchored(&mut first, x, s.3.map(|v| v as f64).unwrap_or(f64::NAN));
            push_gap_anchored(&mut second, x, s.4.map(|v| v as f64).unwrap_or(f64::NAN));
        }
        if start_idx > 0 && !first.is_empty() && first[0][0] > -window_min {
            let prev = &self.samples[start_idx - 1];
            let ay = prev.3.map(|v| v as f64).unwrap_or(f64::NAN);
            let by = prev.4.map(|v| v as f64).unwrap_or(f64::NAN);
            first.insert(0, [-window_min, ay]);
            second.insert(0, [-window_min, by]);
        } else if first.len() == 1 {
            let ay = first[0][1];
            let by = second[0][1];
            first.insert(0, [-0.001, ay]);
            second.insert(0, [-0.001, by]);
        }
        (first, second)
    }
}

fn slim(a: Vec<[f64; 2]>, b: Vec<[f64; 2]>) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    (lttb_downsample(&a, 920), lttb_downsample(&b, 920))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn none_ping_never_becomes_zero() {
        let mut h = NetworkHistory::new();
        h.push(0.0, 1.0, 1.0, None, None);
        h.push(1.0, 2.0, 2.0, Some(20.0), Some(2.0));
        h.push(2.0, 3.0, 3.0, None, None);
        assert_eq!(h.len(), 3);
        let (ping, jitter) = h.latency_series(60);
        assert!(ping.iter().any(|p| p[1] == 20.0));
        assert!(ping.iter().any(|p| p[1].is_nan()));
        assert!(ping.iter().all(|p| p[1] != 0.0));
        assert_eq!(ping.len(), jitter.len());
        assert!(jitter.iter().any(|p| p[1].is_nan()));
    }
    #[test]
    fn per_adapter_series_is_separate() {
        let mut h = NetworkHistory::new();
        for i in 0..5u32 {
            h.push(i as f64, 10.0, 5.0, Some(10.0), Some(1.0));
            h.push_adapter("Eth", i as f64, i as f32, (i * 2) as f32);
            h.push_adapter("Wifi", i as f64, 100.0, 0.0);
        }
        let (rx, tx) = h.traffic_series_scoped(Some("Eth"), 60);
        assert_eq!(rx.len(), 5);
        assert_eq!(rx[4][1], 4.0);
        assert_eq!(tx[4][1], 8.0);
        let (rx2, _) = h.traffic_series_scoped(Some("Wifi"), 60);
        assert_eq!(rx2[4][1], 100.0);
        let (all_rx, _) = h.traffic_series_scoped(None, 60);
        assert_eq!(all_rx.len(), 5);
        assert_eq!(all_rx[4][1], 10.0);
    }
    #[test]
    fn per_adapter_cap_evicts_oldest_only() {
        let mut h = NetworkHistory::new();
        (0..33u32).for_each(|i| h.push_adapter(&format!("a{i}"), i as f64, 1.0, 1.0));
        assert!(h.per_adapter.len() == 32 && !h.per_adapter.contains_key("a0"));
        assert!(h.per_adapter.contains_key("a1") && h.per_adapter.contains_key("a32"));
    }
}

//! Storage throughput history split from model for the 280-line guard.
//! Minutes-ago coordinates with LTTB downsampling to 920 points.
//! Per-disk series mirror NetworkHistory::per_adapter for chart scope.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::modules::memory::downsampling::lttb_downsample;

type SeriesCache = (Option<u32>, u64, usize, Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>);
type DiskSample = (f64, f32, f32);
fn san_f32(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct StorageHistory {
    samples: VecDeque<DiskSample>,
    per_disk: HashMap<u32, VecDeque<DiskSample>>,
    revision: u64,
    cached: Option<SeriesCache>,
}

impl StorageHistory {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
    #[allow(dead_code)]
    pub fn push(&mut self, timestamp: f64, read_mb_s: f32, write_mb_s: f32) {
        if !timestamp.is_finite() || timestamp < 0.0 {
            return;
        }
        if let Some(last) = self.samples.back() {
            if timestamp <= last.0 {
                return;
            }
        }
        let read = if read_mb_s.is_finite() { read_mb_s.max(0.0) } else { 0.0 };
        let write = if write_mb_s.is_finite() { write_mb_s.max(0.0) } else { 0.0 };
        if self.samples.len() == 900 {
            self.samples.pop_front();
        }
        self.samples.push_back((timestamp, read, write));
        self.revision = self.revision.wrapping_add(1);
        self.cached = None;
    }
    #[allow(dead_code)]
    pub fn push_disk(&mut self, disk: u32, timestamp: f64, read_mb_s: f32, write_mb_s: f32) {
        if !timestamp.is_finite() || timestamp < 0.0 {
            return;
        }
        if self.per_disk.len() >= 32 && !self.per_disk.contains_key(&disk) {
            if let Some(&victim) = self.per_disk.keys().find(|&k| *k != disk) {
                self.per_disk.remove(&victim);
            }
        }
        let entry = self.per_disk.entry(disk).or_default();
        if let Some(last) = entry.back() {
            if timestamp <= last.0 {
                return;
            }
        }
        let read = if read_mb_s.is_finite() { read_mb_s.max(0.0) } else { 0.0 };
        let write = if write_mb_s.is_finite() { write_mb_s.max(0.0) } else { 0.0 };
        if entry.len() == 900 {
            entry.pop_front();
        }
        entry.push_back((timestamp, read, write));
        self.revision = self.revision.wrapping_add(1);
        self.cached = None;
    }
    #[allow(dead_code)]
    #[allow(clippy::type_complexity)]
    pub fn series_points(
        &mut self,
        window_secs: usize,
    ) -> (Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>) {
        self.series_points_scoped(None, window_secs)
    }
    #[allow(dead_code)]
    #[allow(clippy::type_complexity)]
    pub fn series_points_scoped(
        &mut self,
        disk: Option<u32>,
        window_secs: usize,
    ) -> (Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>) {
        if let Some((d, rev, win, r, w)) = self.cached.clone() {
            if rev == self.revision && win == window_secs && d == disk {
                return (r, w);
            }
        }
        let (read, write) = self.build_series_scoped(disk, window_secs);
        let out = (Arc::new(read), Arc::new(write));
        self.cached =
            Some((disk, self.revision, window_secs, Arc::clone(&out.0), Arc::clone(&out.1)));
        out
    }
    #[allow(dead_code)]
    fn build_series(&self, window_secs: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        self.build_series_scoped(None, window_secs)
    }
    #[allow(dead_code)]
    fn build_series_scoped(
        &self,
        disk: Option<u32>,
        window_secs: usize,
    ) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        let samples = match disk {
            Some(d) => match self.per_disk.get(&d) {
                Some(s) => s,
                None => return (Vec::new(), Vec::new()),
            },
            None => &self.samples,
        };
        if samples.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let latest = samples.back().map(|s| s.0).unwrap_or(0.0);
        let win = window_secs as f64;
        let window_min = win / 60.0;
        let mut start_idx = 0;
        for (i, s) in samples.iter().enumerate() {
            if latest - s.0 <= win {
                start_idx = i;
                break;
            }
        }
        let mut read = Vec::new();
        let mut write = Vec::new();
        for s in samples.iter().skip(start_idx) {
            let x = -(latest - s.0) / 60.0;
            read.push([x, san_f32(s.1) as f64]);
            write.push([x, san_f32(s.2) as f64]);
        }
        if start_idx > 0 && !read.is_empty() && read[0][0] > -window_min {
            let prev = samples[start_idx - 1];
            read.insert(0, [-window_min, san_f32(prev.1) as f64]);
            write.insert(0, [-window_min, san_f32(prev.2) as f64]);
        } else if read.len() == 1 {
            let ry = read[0][1];
            let wy = write[0][1];
            read.insert(0, [-0.001, ry]);
            write.insert(0, [-0.001, wy]);
        }
        (lttb_downsample(&read, 920), lttb_downsample(&write, 920))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_push_and_window() {
        let mut h = StorageHistory::new();
        assert!(h.is_empty());
        for i in 0..5u32 {
            h.push(i as f64, i as f32, (i * 10) as f32);
        }
        assert_eq!(h.len(), 5);
        let (r, w) = h.series_points(60);
        assert_eq!(r.len(), 5);
        assert_eq!(w.len(), 5);
        assert_eq!(r[0][1], 0.0);
        assert_eq!(r[4][1], 4.0);
        assert_eq!(r[4][0], 0.0);
        assert!(r[0][0] < r[4][0]);
        let (r2, _) = h.series_points(2);
        assert_eq!(r2.len(), 3);
        assert_eq!(r2[0][1], 2.0);
        let (re, _) = StorageHistory::new().series_points(60);
        assert!(re.is_empty());
        let (r3, _) = h.series_points(60);
        assert_eq!(r3.len(), 5);
        assert_eq!(r3[0][1], 0.0);
    }
    #[test]
    fn history_evicts_oldest_past_900() {
        let mut h = StorageHistory::new();
        for i in 0..905u32 {
            h.push(i as f64, i as f32, 1.0);
        }
        assert_eq!(h.len(), 900);
        let (r, _) = h.series_points(900);
        assert_eq!(r.len(), 900);
        assert_eq!(r[899][1], 904.0);
    }
    #[test]
    fn per_disk_series_is_separate() {
        let mut h = StorageHistory::new();
        for i in 0..5u32 {
            h.push(i as f64, 10.0, 5.0);
            h.push_disk(0, i as f64, i as f32, (i * 2) as f32);
            h.push_disk(1, i as f64, 100.0, 0.0);
        }
        let (r, w) = h.series_points_scoped(Some(0), 60);
        assert_eq!(r.len(), 5);
        assert_eq!(r[4][1], 4.0);
        assert_eq!(w[4][1], 8.0);
        let (r1, _) = h.series_points_scoped(Some(1), 60);
        assert_eq!(r1[4][1], 100.0);
        let (all, _) = h.series_points_scoped(None, 60);
        assert_eq!(all[4][1], 10.0);
        let (missing, _) = h.series_points_scoped(Some(7), 60);
        assert!(missing.is_empty());
    }
}

//! Storage throughput history split from model for the 280-line guard.
//! Minutes-ago coordinates with LTTB downsampling to 920 points.
//! Per-disk series mirror NetworkHistory::per_adapter for chart scope.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::core::downsampling::lttb_downsample;
use crate::core::ring_history::{is_newer, window_bounds, HISTORY_CAPACITY};

type SeriesCache = (Option<u32>, u64, usize, Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>);
type DiskSample = (f64, f32, f32);

fn san_f32(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

fn push_gap_anchored(pts: &mut Vec<[f64; 2]>, x: f64, y: f64) {
    if let Some(&[px, _]) = pts.last() {
        if x - px > 4.5 / 60.0 {
            pts.extend([[px + 0.01, 0.0], [x - 0.01, 0.0]]);
        }
    }
    pts.push([x, y]);
}

#[derive(Clone, Debug, Default)]
pub struct StorageHistory {
    samples: VecDeque<DiskSample>,
    per_disk: HashMap<u32, VecDeque<DiskSample>>,
    per_disk_order: VecDeque<u32>,
    revision: u64,
    cached: Option<SeriesCache>,
}

impl StorageHistory {
    #[cfg(test)]
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
    pub fn push(&mut self, timestamp: f64, read_mb_s: f32, write_mb_s: f32) {
        if !timestamp.is_finite() || timestamp < 0.0 {
            return;
        }
        if !is_newer(self.samples.back().map(|s| s.0), timestamp) {
            return;
        }
        let read = san_f32(read_mb_s);
        let write = san_f32(write_mb_s);
        if self.samples.len() == HISTORY_CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back((timestamp, read, write));
        self.revision = self.revision.wrapping_add(1);
        self.cached = None;
    }
    pub fn push_disk(&mut self, disk: u32, timestamp: f64, read_mb_s: f32, write_mb_s: f32) {
        if !timestamp.is_finite() || timestamp < 0.0 {
            return;
        }
        if !self.per_disk.contains_key(&disk) {
            crate::util::insert_capped_fifo(
                &mut self.per_disk,
                &mut self.per_disk_order,
                32,
                disk,
                VecDeque::new(),
            );
        }
        let Some(entry) = self.per_disk.get_mut(&disk) else { return };
        if !is_newer(entry.back().map(|s| s.0), timestamp) {
            return;
        }
        let read = san_f32(read_mb_s);
        let write = san_f32(write_mb_s);
        if entry.len() == HISTORY_CAPACITY {
            entry.pop_front();
        }
        entry.push_back((timestamp, read, write));
        self.revision = self.revision.wrapping_add(1);
        self.cached = None;
    }
    #[cfg(test)]
    #[allow(clippy::type_complexity)]
    pub fn series_points(
        &mut self,
        window_secs: usize,
    ) -> (Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>) {
        self.series_points_scoped(None, window_secs)
    }
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
    #[cfg(test)]
    pub fn window_peak_scoped(&self, disk: Option<u32>, window_secs: usize) -> f64 {
        let samples = match disk {
            Some(d) => match self.per_disk.get(&d) {
                Some(s) => s,
                None => return 0.0,
            },
            None => &self.samples,
        };
        let Some(latest) = samples.back().map(|s| s.0) else { return 0.0 };
        let win = window_secs as f64;
        let timestamps: VecDeque<f64> = samples.iter().map(|s| s.0).collect();
        let (start, _) = window_bounds(samples.len(), latest, &timestamps, win);
        samples
            .iter()
            .skip(start)
            .fold(0.0_f64, |acc, s| acc.max(san_f32(s.1).max(san_f32(s.2)) as f64))
    }
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
        let timestamps: VecDeque<f64> = samples.iter().map(|s| s.0).collect();
        let (start_idx, _) = window_bounds(samples.len(), latest, &timestamps, win);
        let mut read = Vec::new();
        let mut write = Vec::new();
        for s in samples.iter().skip(start_idx) {
            let x = -(latest - s.0) / 60.0;
            push_gap_anchored(&mut read, x, san_f32(s.1) as f64);
            push_gap_anchored(&mut write, x, san_f32(s.2) as f64);
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
    fn history_evicts_oldest_past_capacity() {
        let mut h = StorageHistory::new();
        for i in 0..1925u32 {
            h.push(i as f64, i as f32, 1.0);
        }
        assert_eq!(h.len(), 1920);
        let (r, _) = h.series_points(2000);
        assert_eq!(r.len(), 920);
        assert_eq!(r.last().unwrap()[1], 1924.0);
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
    #[test]
    fn gap_anchor_inserts_zero_points() {
        let mut h = StorageHistory::new();
        h.push(0.0, 5.0, 1.0);
        h.push(10.0, 7.0, 2.0);
        let (r, w) = h.series_points(60);
        assert_eq!(r.len(), 4);
        assert_eq!(w.len(), 4);
        assert_eq!(r[1][1], 0.0);
        assert_eq!(r[2][1], 0.0);
        assert_eq!(w[1][1], 0.0);
        assert_eq!(w[2][1], 0.0);
    }
    #[test]
    fn window_scoped_peak_uses_max_not_sum() {
        let mut h = StorageHistory::new();
        for i in 0..10u32 {
            h.push(i as f64, 300.0, 300.0);
        }
        assert_eq!(h.window_peak_scoped(None, 60), 300.0);
        assert_eq!(h.window_peak_scoped(None, 300), 300.0);
        assert_eq!(h.window_peak_scoped(None, 900), 300.0);
        h.push(100.0, 500.0, 100.0);
        assert_eq!(h.window_peak_scoped(None, 5), 500.0);
        assert_eq!(h.window_peak_scoped(None, 60), 500.0);
    }
    #[test]
    fn per_disk_cap_evicts_earliest() {
        let mut h = StorageHistory::new();
        (0..33u32).for_each(|i| h.push_disk(i, i as f64, 1.0, 1.0));
        assert!(h.per_disk.len() == 32 && !h.per_disk.contains_key(&0));
        assert!(h.per_disk.contains_key(&1) && h.per_disk.contains_key(&32));
    }
}

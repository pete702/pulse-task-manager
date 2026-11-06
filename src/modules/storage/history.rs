//! Storage throughput history split from model for the 280-line guard.
//! Minutes-ago coordinates with LTTB downsampling to 300 points.

use std::collections::VecDeque;
use std::sync::Arc;

use crate::modules::memory::downsampling::lttb_downsample;

type SeriesCache = (u64, usize, Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>);

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct StorageHistory {
    samples: VecDeque<(f64, f32, f32)>,
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
        if self.samples.len() == 900 {
            self.samples.pop_front();
        }
        self.samples.push_back((timestamp, read_mb_s, write_mb_s));
        self.revision = self.revision.wrapping_add(1);
        self.cached = None;
    }
    #[allow(dead_code, clippy::type_complexity)]
    pub fn series_points(
        &mut self,
        window_secs: usize,
    ) -> (Arc<Vec<[f64; 2]>>, Arc<Vec<[f64; 2]>>) {
        if let Some((rev, win, r, w)) = self.cached.clone() {
            if rev == self.revision && win == window_secs {
                return (r, w);
            }
        }
        let (read, write) = self.build_series(window_secs);
        let out = (Arc::new(read), Arc::new(write));
        self.cached = Some((self.revision, window_secs, Arc::clone(&out.0), Arc::clone(&out.1)));
        out
    }
    #[allow(dead_code)]
    fn build_series(&self, window_secs: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        if self.samples.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let latest = self.samples.back().map(|s| s.0).unwrap_or(0.0);
        let win = window_secs as f64;
        let mut read = Vec::new();
        let mut write = Vec::new();
        for (t, r, w) in &self.samples {
            if latest - t <= win {
                let x = -(latest - t) / 60.0;
                read.push([x, *r as f64]);
                write.push([x, *w as f64]);
            }
        }
        if read.len() == 1 {
            let ry = read[0][1];
            let wy = write[0][1];
            read.insert(0, [-1.0 / 60.0, ry]);
            write.insert(0, [-1.0 / 60.0, wy]);
        }
        (lttb_downsample(&read, 300), lttb_downsample(&write, 300))
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
        assert_eq!(r.len(), 300);
        assert_eq!(r[299][1], 904.0);
    }
}

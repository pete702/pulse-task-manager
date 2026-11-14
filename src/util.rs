//! Utility functions for PULSE.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex, OnceLock};

/// Inserts a key/value into a capped map with FIFO eviction tracked by `order`.
/// Evicts from the front of `order` when at capacity and inserting a new key.
pub fn insert_capped_fifo<K, V>(
    map: &mut HashMap<K, V>,
    order: &mut VecDeque<K>,
    cap: usize,
    key: K,
    val: V,
) where
    K: Clone + Eq + Hash,
{
    if cap == 0 {
        map.clear();
        order.clear();
        return;
    }
    if !map.contains_key(&key) {
        while map.len() >= cap {
            if let Some(old) = order.pop_front() {
                map.remove(&old);
            } else if let Some(any) = map.keys().next().cloned() {
                map.remove(&any);
            } else {
                break;
            }
        }
        order.push_back(key.clone());
    }
    map.insert(key, val);
}

/// Removes a key from a capped FIFO map and keeps its order ring in sync.
pub fn remove_capped_fifo<K, V>(
    map: &mut HashMap<K, V>,
    order: &mut VecDeque<K>,
    key: &K,
) -> Option<V>
where
    K: Eq + Hash,
{
    let removed = map.remove(key);
    if removed.is_some() {
        order.retain(|k| k != key);
    }
    removed
}

type InternPool = (HashMap<String, Arc<str>>, VecDeque<String>);

/// Interns process names so repeated names share one allocation.
pub fn intern_name(raw: &str) -> Arc<str> {
    static INTERN_POOL: OnceLock<Mutex<InternPool>> = OnceLock::new();
    let mut guard = INTERN_POOL
        .get_or_init(|| Mutex::new((HashMap::new(), VecDeque::new())))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let (map, order) = &mut *guard;
    if let Some(v) = map.get(raw) {
        return Arc::clone(v);
    }
    let key = raw.to_string();
    let v: Arc<str> = Arc::from(raw);
    insert_capped_fifo(map, order, 512, key, Arc::clone(&v));
    v
}

/// Returns `v` if finite, otherwise `fallback`.
#[cfg(test)]
pub fn sanitize_f32(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Returns `v` if finite, otherwise `fallback`.
#[cfg(test)]
pub fn sanitize_f64(v: f64, fallback: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Case-insensitive substring check for ASCII and Unicode strings.
/// `needle` is matched case-insensitively (caller may pre-lowercase).
pub fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.is_ascii() && needle.is_ascii() {
        let nb = needle.as_bytes();
        let hb = haystack.as_bytes();
        if nb.len() > hb.len() {
            return false;
        }
        return hb.windows(nb.len()).any(|w| w.eq_ignore_ascii_case(nb));
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Formats uptime in seconds into a human-readable string (days, hours, minutes, seconds).
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 || days > 0 {
        parts.push(format!("{}h", hours));
    }
    if mins > 0 || hours > 0 || days > 0 {
        parts.push(format!("{}m", mins));
    }
    parts.push(format!("{}s", secs));

    parts.join(" ")
}

/// Formats current system time as HH:MM:SS using std::time (no chrono dependency).
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn format_timestamp_now() -> String {
    let mut st: windows_sys::Win32::Foundation::SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe {
        windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut st as *mut _);
    }
    format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond)
}

/// Formats current system time as HH:MM:SS using std::time (no chrono dependency).
#[cfg(not(target_os = "windows"))]
pub fn format_timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_capped_fifo_evicts_earliest_inserted_key() {
        let mut map = HashMap::new();
        let mut order = VecDeque::new();
        insert_capped_fifo(&mut map, &mut order, 2, 1u32, "a");
        insert_capped_fifo(&mut map, &mut order, 2, 2u32, "b");
        insert_capped_fifo(&mut map, &mut order, 2, 3u32, "c");
        assert_eq!(map.len(), 2);
        assert!(!map.contains_key(&1));
        assert_eq!(map.get(&2), Some(&"b"));
        assert_eq!(map.get(&3), Some(&"c"));
        assert_eq!(order.front().copied(), Some(2));
    }

    #[test]
    fn insert_capped_fifo_string_keys_and_existing_update() {
        let mut map = HashMap::new();
        let mut order = VecDeque::new();
        insert_capped_fifo(&mut map, &mut order, 2, "a".to_string(), 1);
        insert_capped_fifo(&mut map, &mut order, 2, "b".to_string(), 2);
        insert_capped_fifo(&mut map, &mut order, 2, "a".to_string(), 10);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a"), Some(&10));
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(order.iter().cloned().collect::<Vec<_>>(), vec!["a", "b"]);
        insert_capped_fifo(&mut map, &mut order, 2, "c".to_string(), 3);
        assert_eq!(map.len(), 2);
        assert!(!map.contains_key("a"));
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(map.get("c"), Some(&3));
        assert_eq!(order.front().map(String::as_str), Some("b"));
    }

    #[test]
    fn remove_capped_fifo_cleans_map_and_order() {
        let mut map = HashMap::new();
        let mut order = VecDeque::new();
        insert_capped_fifo(&mut map, &mut order, 2, "a".to_string(), 1);
        insert_capped_fifo(&mut map, &mut order, 2, "b".to_string(), 2);
        assert_eq!(remove_capped_fifo(&mut map, &mut order, &"a".to_string()), Some(1));
        assert_eq!(map.len(), 1);
        assert_eq!(order.iter().cloned().collect::<Vec<_>>(), vec!["b"]);
        insert_capped_fifo(&mut map, &mut order, 2, "c".to_string(), 3);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("b"));
        assert!(map.contains_key("c"));
        assert!(!map.contains_key("a"));
        assert_eq!(order.front().map(String::as_str), Some("b"));
        insert_capped_fifo(&mut map, &mut order, 2, "d".to_string(), 4);
        assert!(!map.contains_key("b"));
        assert!(map.contains_key("c"));
        assert!(map.contains_key("d"));
    }

    #[test]
    fn contains_ci_ascii_and_unicode() {
        assert!(contains_ci("Chrome.exe", "chrome"));
        assert!(contains_ci("chrome.exe", "CHROM"));
        assert!(contains_ci("chrome.exe", "chrom"));
        assert!(!contains_ci("chrome.exe", "edge"));
        assert!(contains_ci("a", ""));
        assert!(!contains_ci("ab", "abc"));
        assert!(contains_ci("Caf\u{00e9}", "CAF\u{00c9}"));
        assert!(!contains_ci("Caf\u{00e9}", "tea"));
    }
}

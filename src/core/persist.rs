use crate::core::persist_codec;
use crate::modules::cpu::table_state::CpuSortCol;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::{Mutex, OnceLock};
use std::thread;

pub const MIN_WINDOW_WIDTH: f32 = 860.0;
static STAGED_STATE: OnceLock<Mutex<Option<PersistedState>>> = OnceLock::new();
static SAVE_WORKER: OnceLock<Mutex<Option<SaveWorker>>> = OnceLock::new();

/// Owns the async config-save channel and the thread that drains it.
pub struct SaveWorker {
    tx: SyncSender<(PathBuf, String)>,
    handle: Option<thread::JoinHandle<()>>,
}

impl SaveWorker {
    fn spawn() -> Self {
        let (tx, rx) = sync_channel::<(PathBuf, String)>(2);
        let handle = thread::Builder::new()
            .name("pulse_cfg_save".into())
            .spawn(move || {
                while let Ok((p, s)) = rx.recv() {
                    let failed = write_config_file(&p, &s).is_err();
                    WRITE_FAILED.store(failed, Ordering::Relaxed);
                }
            })
            .ok();
        Self { tx, handle }
    }
}

fn save_worker() -> &'static Mutex<Option<SaveWorker>> {
    SAVE_WORKER.get_or_init(|| Mutex::new(None))
}

fn save_sender() -> SyncSender<(PathBuf, String)> {
    let mut guard = save_worker().lock().unwrap_or_else(|e| e.into_inner());
    guard.get_or_insert_with(SaveWorker::spawn).tx.clone()
}

/// Drains queued writes and joins the save thread before the final sync save.
pub fn join_save_worker() {
    if let Some(SaveWorker { tx, handle }) =
        save_worker().lock().unwrap_or_else(|e| e.into_inner()).take()
    {
        drop(tx);
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }
}
static WRITE_LOCK: Mutex<()> = Mutex::new(());
static WRITE_FAILED: AtomicBool = AtomicBool::new(false);
static PORTABLE_MODE: AtomicBool = AtomicBool::new(false);

/// Pure path rule: portable mode uses `pulse.cfg` beside the executable.
pub fn resolve_portable_config_path(
    exe_path: &Path,
    cli_portable: bool,
    local_pulse_cfg_exists: bool,
) -> Option<PathBuf> {
    if cli_portable || local_pulse_cfg_exists {
        exe_path.parent().map(|dir| dir.join("pulse.cfg"))
    } else {
        None
    }
}

/// Detects portable mode from `--portable` or a side-by-side `pulse.cfg`.
pub fn init_portable_mode(cli_portable: bool) {
    let local_exists = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("pulse.cfg").is_file()))
        .unwrap_or(false);
    PORTABLE_MODE.store(cli_portable || local_exists, Ordering::SeqCst);
}

pub fn is_portable_mode() -> bool {
    PORTABLE_MODE.load(Ordering::SeqCst)
}

fn with_staged<T>(f: impl FnOnce(&mut Option<PersistedState>) -> T) -> T {
    f(&mut STAGED_STATE.get_or_init(|| Mutex::new(None)).lock().unwrap_or_else(|e| e.into_inner()))
}
pub fn stage_state(s: &PersistedState) {
    with_staged(|x| *x = Some(s.clone()));
}
pub fn clear_staged_state() {
    with_staged(|x| *x = None);
}
pub fn flush_staged_state() {
    if let Some(s) = with_staged(|x| x.take()) {
        save_now(&s);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersistedState {
    pub active_tab: u8,
    pub group_clones: bool,
    pub compact_rows: bool,
    pub sort_col: CpuSortCol,
    pub sort_asc: bool,
    pub cpu_show_grid: bool,
    pub mem_window_secs: u32,
    pub cpu_window_secs: u32,
    pub storage_window_secs: u64,
    pub network_window_secs: u64,
    pub network_chart_mode: u8,
    pub network_probe_enabled: bool,
    pub process_quick_filter: u8,
    pub global_hotkey_enabled: bool,
    pub sidebar_hidden: bool,
    pub win_w: f32,
    pub win_h: f32,
    pub cpu_col_widths: Option<[f32; 7]>,
}
impl Default for PersistedState {
    fn default() -> Self {
        Self {
            active_tab: 0,
            group_clones: true,
            compact_rows: true,
            sort_col: CpuSortCol::default(),
            sort_asc: false,
            cpu_show_grid: false,
            mem_window_secs: 300,
            cpu_window_secs: 300,
            storage_window_secs: 300,
            network_window_secs: 300,
            network_chart_mode: 0,
            network_probe_enabled: true,
            process_quick_filter: 0,
            global_hotkey_enabled: true,
            sidebar_hidden: false,
            win_w: 1180.0,
            win_h: 760.0,
            cpu_col_widths: None,
        }
    }
}

pub fn state_path() -> Option<PathBuf> {
    if is_portable_mode() {
        let exe = std::env::current_exe().ok()?;
        return resolve_portable_config_path(&exe, true, true);
    }
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("PULSE").join("state.cfg"))
}
pub fn load_state() -> PersistedState {
    let Some(path) = state_path() else { return PersistedState::default() };
    let Ok(f) = std::fs::File::open(&path) else { return PersistedState::default() };
    let mut buf = Vec::new();
    if f.take(64 * 1024).read_to_end(&mut buf).is_err() {
        return PersistedState::default();
    }
    String::from_utf8(buf).map(|t| parse(&t)).unwrap_or_default()
}
fn is_sym(p: &Path) -> bool {
    std::fs::symlink_metadata(p).map(|m| m.file_type().is_symlink()).unwrap_or(false)
}
fn ensure_tmp_clean(tmp: &Path) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind as K};
    match std::fs::symlink_metadata(tmp) {
        Ok(m) if m.file_type().is_symlink() => Err(Error::new(K::PermissionDenied, "symlink")),
        Ok(m) if !m.is_file() => Err(Error::new(K::AlreadyExists, "tmp collision")),
        Ok(_) => std::fs::remove_file(tmp),
        Err(e) if e.kind() == K::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
pub fn write_config_file(path: &PathBuf, payload: &str) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind as K};
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let parent = path.parent().ok_or_else(|| Error::new(K::InvalidInput, "no parent"))?;
    if is_sym(parent) || is_sym(path) {
        return Err(Error::new(K::PermissionDenied, "symlink"));
    }
    std::fs::create_dir_all(parent)?;
    if is_sym(parent) || is_sym(path) {
        return Err(Error::new(K::PermissionDenied, "symlink"));
    }
    let tmp = path.with_file_name("state.cfg.tmp");
    ensure_tmp_clean(&tmp)?;
    let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    if let Err(e) = f.write_all(payload.as_bytes()).and_then(|_| f.sync_all()) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    drop(f);
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}
pub fn save_now(s: &PersistedState) {
    if let Some(path) = state_path() {
        if write_config_file(&path, &persist_codec::serialize(s)).is_err() {
            WRITE_FAILED.store(true, Ordering::Relaxed);
        } else {
            WRITE_FAILED.store(false, Ordering::Relaxed);
        }
    }
    clear_staged_state();
}
/// Persists the global hotkey toggle without disturbing other settings.
pub fn save_hotkey_enabled(enabled: bool) {
    let mut s = load_state();
    s.global_hotkey_enabled = enabled;
    crate::platform::hotkey::set_global_hotkey_enabled(enabled);
    save_now(&s);
}
pub fn save_if_due(s: &PersistedState) -> bool {
    let Some(path) = state_path() else { return true };
    match save_sender().try_send((path, persist_codec::serialize(s))) {
        Ok(()) => {
            clear_staged_state();
            true
        }
        Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => false,
    }
}
pub fn config_write_failed() -> bool {
    WRITE_FAILED.load(Ordering::Relaxed)
}
#[allow(unused_imports)]
pub use crate::core::persist_codec::{
    active_tab_from_index, active_tab_index, chart_window_from_secs, parse, parse_col_widths,
    secs_from_chart_window, serialize,
};

#[cfg(test)]
mod portable_tests {
    use super::*;

    #[test]
    fn portable_path_from_cli_flag() {
        let exe = Path::new("C:\\apps\\pulse_manager.exe");
        assert_eq!(
            resolve_portable_config_path(exe, true, false),
            Some(PathBuf::from("C:\\apps\\pulse.cfg"))
        );
    }

    #[test]
    fn portable_path_from_side_by_side_cfg() {
        let exe = Path::new("C:\\apps\\pulse_manager.exe");
        assert_eq!(
            resolve_portable_config_path(exe, false, true),
            Some(PathBuf::from("C:\\apps\\pulse.cfg"))
        );
    }

    #[test]
    fn portable_path_falls_back_to_appdata() {
        let exe = Path::new("C:\\apps\\pulse_manager.exe");
        assert_eq!(resolve_portable_config_path(exe, false, false), None);
    }
}

#[cfg(test)]
#[path = "persist_tests.rs"]
mod tests;

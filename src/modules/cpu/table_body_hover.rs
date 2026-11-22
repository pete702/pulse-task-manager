//! Pure helpers for CPU table row hover and copy text.
use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::proc_meta::ProcMeta;

/// Returns the executable path and lazily cached metadata for a display row.
pub(crate) fn row_meta<'a>(
    item: &ProcessCpuItem,
    raw_rows: &'a [RawProcessCpuRow],
) -> (&'a str, ProcMeta) {
    if let Some(r) = raw_rows.iter().find(|r| r.pid == item.pid && r.start_time == item.start_time)
    {
        let path = r.exe_path.as_deref().unwrap_or("");
        let meta = crate::modules::cpu::proc_meta::get_or_cache(
            item.pid,
            item.start_time,
            r.command_line.as_deref().unwrap_or(""),
            r.user.as_deref().unwrap_or(""),
        );
        (path, meta)
    } else {
        let meta = crate::modules::cpu::proc_meta::get_or_cache(item.pid, item.start_time, "", "");
        ("", meta)
    }
}

/// Builds the multi-line tooltip shown when hovering a process row.
pub(crate) fn hover_text(item: &ProcessCpuItem, raw_rows: &[RawProcessCpuRow]) -> String {
    let (path, meta) = row_meta(item, raw_rows);
    let known = if path.is_empty() { None } else { Some(path) }
        .or_else(|| (!item.exe_path.is_empty()).then(|| item.exe_path.as_ref()));
    let exe = crate::modules::cpu::exe_cache::resolve_exe_cached(item.pid, item.start_time, known);
    let cmd = if meta.command_line.is_empty() { "-" } else { meta.command_line.as_ref() };
    let usr = if meta.user.is_empty() { "-" } else { meta.user.as_ref() };
    format!("{}\n{}\nCommandLine: {}\nUser: {}", item.name, exe, cmd, usr)
}

/// Builds the extended Ctrl+C summary for a process row.
pub(crate) fn copy_summary(item: &ProcessCpuItem, raw_rows: &[RawProcessCpuRow]) -> String {
    let (path, meta) = row_meta(item, raw_rows);
    crate::modules::cpu::proc_meta::format_process_copy_line(
        &item.name,
        item.pid,
        &item.ram_text,
        &item.cpu_text,
        path,
        &meta.command_line,
    )
}

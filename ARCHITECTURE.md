# ARCHITECTURE.md — PULSE Project Immutable Laws

> This file defines the **constitution** of the codebase. Any deviation requires an explicit architect decision and an update to this document.

---

## 1. Modularity (Module Boundaries)

| Rule | Value |
|------|-------|
| **Max lines per file** | **180** (hard limit; legacy files under incremental split) |
| **Module structure** | `src/modules/<name>/` with mandatory split: |
| | • `model.rs` — data, pure logic, **unit tests** (`#[cfg(test)]`) |
| | • `collector.rs` — OS data collection (Windows API, `sysinfo`) |
| | • `view.rs` — UI (egui/eframe) |
| **Forbidden** | Mixing collection logic with UI, moving business logic into `view.rs` |
| **Public module API** | Exported **only** via module's `mod.rs` (`pub use model::*; pub use collector::*; pub use view::*;`) |

> **Why 180 lines?** Guarantees: 30-second readability, easy code review, isolated unit tests, no "god files".

---

## 2. Documentation & Comments (Documentation Standard)

| Element | Requirement |
|---------|-------------|
| **Public structs** | `///` in English — purpose, invariants, usage example |
| **Public fields** | `///` — semantics, units, valid ranges |
| **Public functions/methods** | `///` — contract: what it does, arguments, return, errors, side-effects |
| **Private elements** | Optional, but **required** for complex logic |
| **Language** | **English only** for doc-comments (`///`). Code comments (`//`) — English permitted. |

> **Example:**
> ```rust
> /// System physical memory: total, available, used.
> /// All values in **bytes**.
> #[derive(Debug, Clone, Copy, PartialEq, Eq)]
> pub struct PhysicalMemory {
>     /// Total physical RAM (bytes).
>     pub total: u64,
>     /// Available for allocation without swapping (bytes).
>     pub available: u64,
>     /// Current usage (bytes).
>     pub used: u64,
> }
> ```

---

## 3. Pinned Dependencies

> **FORBIDDEN** to use `^`, `~`, `*`, or `latest`. Versions are **hard-pinned** for 100% reproducible builds and API compatibility.

```toml
[dependencies]
eframe = "0.27.2"
egui = "0.27.2"
egui_plot = "0.27.2"
egui_extras = "0.27.2"
sysinfo = "0.30.12"
windows-sys = { version = "0.52.0", features = ["Win32_System_ProcessStatus", "Win32_System_Threading", "Win32_Foundation", "Win32_Globalization"] }
```

| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` | 0.27.2 | Native egui framework (window, event loop, wgpu) |
| `egui` | 0.27.2 | Immediate-mode GUI |
| `egui_plot` | 0.27.2 | Real-time graphs (CPU/RAM/GPU load) |
| `egui_extras` | 0.27.2 | Table, Strip, Image, RetainedImage |
| `sysinfo` | 0.30.12 | Cross-platform system info (CPU, RAM, Disks, Processes) |
| `windows-sys` | 0.52.0 | Direct Windows API (PSAPI, Threading, Foundation) for EmptyWorkingSet, precise metrics |

> **Version updates** — only via ARCHITECTURE.md + CHANGELOG.md + `cargo check` / `cargo test` validation.

---

## 4. "Obsidian HUD" Design System (Design Tokens)

| Token | Value | Application |
|-------|-------|-------------|
| `BG_WINDOW` | `#0B0C10` | Main window background (`CentralPanel`, `SidePanel`) |
| `BG_CARD` | `#12141C` | Card/panel background (process lists, graphs) |
| `BORDER` | `#1F2330` | Card borders, table dividers (1px) |
| `RADIUS` | `6.0` | Corner rounding `CornerRadius::same(6.0)` |
| `MONO_FONT` | `FontId::monospace(size)` | **All numbers**: MB, GB, %, PID, Hz, °C — **monospace only** |

### Traffic-Light Load Colors

| Range | Color | Hex | Name |
|-------|-------|-----|------|
| `< 60%` | ![#00F0FF](https://via.placeholder.com/10/00F0FF/000000?text=+) | `#00F0FF` | **Cyan** — normal |
| `60–85%` | ![#FFB800](https://via.placeholder.com/10/FFB800/000000?text=+) | `#FFB800` | **Amber** — warning |
| `≥ 85%` | ![#FF2A54](https://via.placeholder.com/10/FF2A54/000000?text=+) | `#FF2A54` | **Neon Red** — critical |

> **Rule:** Color is determined by **current percentage** of resource (RAM, CPU, GPU, Disk). Applied to: percentage text, progress bar fill, graph points.

---

## 5. Definition of Done (DoD) — 8 Iron Rules of Step Completion

> **The agent MUST perform ALL 8 actions** on every version completion. No exceptions.

| # | Action | Command / Description | Success Criteria |
|---|--------|----------------------|------------------|
| **1** | **Format** | `cargo fmt` | Zero formatting diffs |
| **2** | **Clippy** | `cargo clippy --all-targets -- -D warnings` | **0 warnings**, **0 errors** |
| **3** | **Unit Tests** | `cargo test` | **All tests pass** (min 3–5 new per step, including i18n coverage) |
| **4** | **Self-Test** | `cargo run -- --self-test` | Prints `SELF-TEST PASSED`, auto-switches tabs & lang, exits 0 |
| **5** | **Build & Export** | `cargo build --release` → copy to `../PULSE_BUILDS/pulse_vX.Y.Z.exe` + `PULSE_LATEST.exe` | Both `.exe` files exist and run |
| **6** | **Clean Backup** | Copy `src/`, `Cargo.toml`, `Cargo.lock`, `*.md` to `../PULSE_BACKUPS/vX.Y.Z_verified/` (NO `target/`, NO `.git/`) | Backup folder complete |
| **7** | **Documentation** | Update `CHANGELOG.md` (EN), append to `../PULSE_AI_CONTEXT.md` (UA), update `../PULSE_UA_MANUAL.md` (UA) — **never delete history** | All three files current |
| **8** | **Git Push** | `git add . && git commit -m "vX.Y.Z: description" && git push` | Remote `main` updated |

### Iron Testing Standard (Mandatory for Every Step)

- **Every step** (including visual `view.rs`, `chart_view.rs`, `table_view.rs`) adds **minimum 3–5 new unit tests** covering:
  1. **Math logic** (pure functions, edge cases, saturating arithmetic, clamping)
  2. **i18n translations** — both `Lang::Uk` and `Lang::En` for all new keys
  3. **UI state transitions** — tab switching, language toggling, snapshot handling
- **Live self-test** (`cargo run -- --self-test`) must **auto-switch all available tabs and languages** (`Lang::Uk` → `Lang::En`) during test frames before auto-close to guarantee zero panics in rendering.

---

## 6. Project Roadmap — 7 Phases to v1.0.0

| Phase | Versions | Focus | Status |
|-------|----------|-------|--------|
| **1. Foundation & Infrastructure** | v0.0.1–v0.0.3 | Git, Cargo, Obsidian HUD theme, background worker (500ms), i18n with automatic OS language detection, system fonts, generated icon, self-test, CI, backup, self-diagnosis badge | ✅ **Done** |
| **2. Memory Module** | v0.0.4–v0.0.8 | Pure math model (`MemorySnapshot`, `MemoryHistory` 900-point ring buffer), `sysinfo` collector, `K32GetPerformanceInfo` kernel metrics, RAM/Swap cards, progress bars, multi-horizon (1m/5m/15m) `egui_plot` history, auto-detect Windows UI Language, worker optimization (refresh_pid), self working-set trim | ✅ **Done** (v0.0.8) |
| **3. Process Memory Model & Table** | v0.0.8–v0.0.9 | v0.0.8: `ProcessMemoryItem`, clone grouping, `is_hog` detector, `MemorySpikeEvent` black box, process memory scan; v0.0.9: Mission Center-style Memory sub-view switcher, virtualized `egui_extras::TableBuilder` (O(visible_rows)), instant name/PID search, `[Nx]` clone grouping, sortable columns, `⚠️ HOG` badges, spike banner | ✅ **Done** (v0.0.9) |
| **4. CPU Module** | v0.1.0–v0.2.0 | Per-core usage, frequency, temperature (MSR), `egui_plot` graphs, top consumers table | ⏳ Planned |
| **5. Storage I/O Module** | v0.2.0–v0.3.0 | Disk read/write throughput, latency, queue depth, SMART health, partition usage | ⏳ Planned |
| **6. Network & GPU Module** | v0.3.0–v0.5.0 | Per-interface throughput, TCP/UDP stats, gamer ping, DXGI GPU usage/VRAM/temp | ⏳ Planned |
| **7. Mini HUD Overlay** | v0.5.0–v1.0.0 | Always-on-top overlay, hotkey toggle, customizable panels, export to HTML/JSON | 🎯 **Final Target** |

---

## 7. Project Structure (Reference Tree)

```
pulse_manager/
├── .gitignore
├── LICENSE
├── README.md
├── ARCHITECTURE.md
├── CHANGELOG.md
├── Cargo.toml
├── Cargo.lock
├── .github/workflows/ci.yml
├── src/
│   ├── main.rs                 # Entry point: eframe::run_native
│   ├── app.rs                  # Root App struct, state, Memory sub-view, update loop
│   ├── theme.rs                # Obsidian HUD tokens + style builder + fonts
│   ├── i18n.rs                 # UI language auto-detected from Windows (GetUserDefaultUILanguage)
│   ├── core/
│   │   ├── mod.rs              # SystemSnapshot, mpsc channel types, spawn_telemetry_worker
│   │   └── worker.rs           # Background thread: 1000 ms interval, process scan + self-trim + self RSS → channel
│   ├── ui/
│   │   ├── mod.rs              # UI chrome exports
│   │   └── chrome.rs           # Top panel, sidebar, status bar rendering
│   ├── modules/
│   │   ├── mod.rs              # Pub re-exports all modules
│   │   ├── memory/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs        # MemorySnapshot, MemoryHistory (900 pts / 15 min), ChartWindow
│   │   │   ├── process_model.rs # ProcessMemoryItem, clone grouping, sorting, MemorySpikeEvent
│   │   │   ├── collector.rs    # sysinfo + windows-sys (K32GetPerformanceInfo, K32GetProcessMemoryInfo, K32EmptyWorkingSet)
│   │   │   ├── view.rs         # RAM/Swap cards, kernel mini-tiles, localized labels
│   │   │   ├── chart_view.rs   # Multi-horizon (1m/5m/15m) dual-line neon chart (egui_plot)
│   │   │   ├── page.rs         # Memory sub-view switcher (Overview / Processes) + compact RAM bar
│   │   │   ├── table_view.rs   # MemoryTableState + search/group/sort orchestration
│   │   │   ├── table_body.rs   # Virtualized TableBuilder rows + sortable header (O(visible_rows))
│   │   │   └── table_widgets.rs # Spike banner, toolbar, [Nx]/⚠️ HOG badges, byte formatting
│   │   ├── processes/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs        # ProcessEntry, GroupKey, ProcessGroup
│   │   │   ├── collector.rs    # sysinfo::Processes + windows-sys details
│   │   │   └── view.rs         # Table, grouping, search, monospace PID
│   │   └── system/
│   │       ├── mod.rs
│   │       ├── model.rs        # CpuInfo, GpuInfo, DiskInfo, NetInfo
│   │       ├── collector.rs    # sysinfo collectors
│   │       └── view.rs         # System overview panels
│   └── utils/
│       ├── mod.rs
│       ├── formatting.rs       # bytes_to_human, monospace helpers
│       └── windows_api.rs      # Wrappers for EmptyWorkingSet, PSAPI
└── tests/                      # Integration tests (optional)
```

---

## 8. Forbidden Patterns (Anti-Patterns)

| Pattern | Why Forbidden | Alternative |
|---------|---------------|-------------|
| `unwrap()` / `expect()` in production code | Panic = UI crash | `Result` + `?` / `tracing::error!` + fallback |
| Global `static mut` / `lazy_static` | Unsafe, hard to test | Dependency Injection via `App` state |
| Blocking calls in UI thread | Frame freeze (jank) | `tokio::task::spawn_blocking` / rayon / channels |
| Hardcoded colors/sizes in `view.rs` | Theme inconsistency | `theme.rs` tokens + `Theme::apply(&mut ctx)` |
| Module without `model.rs`/`collector.rs`/`view.rs` | Violates separation of concerns | Create three files or merge with another module |

---

## 9. CI/CD Gates (Pre-Push Checks)

```yaml
# .github/workflows/ci.yml
- cargo fmt --check
- cargo clippy --all-targets -- -D warnings
- cargo check --all-targets
- cargo test --all-targets
- cargo build --release
- cargo run -- --self-test
```

> All gates **mandatory**. Merge without green checks — forbidden.

---

## 10. Virtualized Process Table (v0.0.9)

### 10.1 Mission Center Sub-view Pattern (`page.rs`)

The Memory tab is a **page** with two sub-views, not two separate tabs:

| Element | Contract |
|---------|----------|
| `MemorySubView` | `Overview` (default) or `Processes`. Pure `Copy` enum with no UI state. |
| `MemorySubView::label(lang, process_count)` | Localized switcher text with the live process count: `◈ Telemetry Overview` / `📋 Memory Processes (N)` in English; the Ukrainian dictionary supplies equivalent localized strings at runtime. |
| Switching | The switcher only mutates `*sub_view`; each branch renders an independent composition of existing modules, so no state is lost when switching. |
| Visual rule | Active sub-view button uses `ACCENT_CYAN.gamma_multiply(0.15)` fill and `TEXT_PRIMARY`; inactive uses `BG_CARD` and `TEXT_SECONDARY`, inside `card_frame()`. |

### 10.2 Table State (`table_view.rs`)

`MemoryTableState` is the single source of truth for interaction:

| Field | Default | Purpose |
|-------|---------|---------|
| `search_query: String` | `""` | Live search box; case-insensitive name substring or exact PID match. |
| `group_clones: bool` | `true` | Aggregates processes with the same name. |
| `sort_col: MemorySortCol` | `RamBytes` | Active sort column. |
| `sort_asc: bool` | `false` | Descending by default (heaviest first). |
| `selected_pid: Option<u32>` | `None` | Row selection; representative PID for grouped rows. |

`render_memory_table()` calls the pure `filter_group_and_sort()` from `process_model.rs`, then renders the spike banner (if `snap.latest_spike` is set), the toolbar, and the table. The data layer is pure and unit-tested; the view layer contains no filtering logic.

### 10.3 Data Pipeline (Pure `filter_group_and_sort`)

1. **Filter** — keep rows where `search` is empty, equals the PID string, or is a case-insensitive substring of the name.
2. **Group clones** (optional) — sum `ram_bytes` / `virtual_bytes`, increment `process_count`, recalculate `ram_pct`, re-evaluate `is_hog`, and keep the **lowest PID** as the representative.
3. **Sort** — by name, PID, physical RAM, virtual memory, or RAM %, ascending or descending.

Cost: O(n log n) for the sort over all rows, independent of the paint cost.

### 10.4 Virtualized Rendering (`table_body.rs`)

`TableBuilder` with `.body(|body| body.rows(24.0, rows.len(), …))` renders **only visible rows**; the UI cost is O(visible_rows), not O(process count). Columns are name / PID / Physical RAM / Virtual / RAM %, all numeric values use `.monospace()`, and RAM cells use `load_color(ram_pct)` for the traffic-light rule.

Badges rendered next to the name:

- `[Nx]` — cyan badge when `process_count > 1` (clone group size).
- `⚠️ HOG` — red badge when `is_hog` (RAM ≥ 15% of total).

### 10.5 File Split (all ≤ 180 lines)

| File | Responsibility | Lines |
|------|----------------|-------|
| `page.rs` | Sub-view enum, switcher, branch composition, compact RAM bar | 158 |
| `table_view.rs` | `MemoryTableState`, orchestration, empty state | 129 |
| `table_body.rs` | Virtualized rows, sortable header, name cell | 175 |
| `table_widgets.rs` | Spike banner, toolbar, localized labels, byte formatting | 156 |

---

## 11. Working Set Monitoring & Safety-Gated Reclamation

### 11.1 Data Carried by the Snapshot

`MemorySnapshot` now carries the process table and the latest spike:

- `processes: Vec<RawProcessRow>` — `(PID, name, RAM bytes, virtual bytes)`.
- `latest_spike: Option<MemorySpikeEvent>` — the most recent level-triggered spike (RAM ≥ 85%), with timestamp, peak %, culprit name/PID/bytes.

`collect_process_memory()` refreshes with `ProcessRefreshKind::new().with_memory()` and never performs a full system scan; the worker attaches the result to `memory.processes` before sending the snapshot.

### 11.2 Own Working Set (`K32GetProcessMemoryInfo` + `K32EmptyWorkingSet`)

PULSE measures and trims **its own** process only:

| Step | API | Notes |
|------|-----|-------|
| Trim | `K32EmptyWorkingSet(GetCurrentProcess())` | Releases font/OpenGL buffers back to the OS. Called by the worker before measuring, and by the app after OpenGL startup (frame 3) and every 30 frames. |
| Measure | `K32GetProcessMemoryInfo(GetCurrentProcess(), …)` → `WorkingSetSize` | `own_working_set_mb()` returns MiB; used for the self-diagnosis badge. |

All Win32 calls are narrow wrappers in `collector.rs`; `main.rs` keeps `#![deny(unsafe_code)]` and the wrappers carry a local `#[allow(unsafe_code)]` with a documented SAFETY comment.

### 11.3 Safety Gate for Future Per-Process Actions

Before PULSE ever calls `OpenProcess` / `K32EmptyWorkingSet` on **another** process (v0.1.0), the action must pass this gate:

1. Reject PID `0` (`System Idle Process`) and PID `4` (`System`).
2. Reject PULSE's own PID.
3. Reject critical OS processes (`smss.exe`, `csrss.exe`, `wininit.exe`, `services.exe`, `lsass.exe`, `winlogon.exe`).
4. Require an explicit row selection and a confirmation step before the Win32 call.

### 11.4 Reclamation Audit Contract

Every reclamation action must record `(pid, before_bytes, after_bytes, freed_bytes)` by reading the working set immediately before and after the trim, so the UI can report the exact reclaimed amount instead of an estimate.

---

## 12. NVIDIA Agent Skills Engineering Patterns

| NVIDIA Agent Skill | PULSE Pattern | Where | Status (v0.0.9) |
|--------------------|---------------|-------|-----------------|
| `ethos-orchestrate` / `mcore-testing` | Self-Enforcing Architecture Guard — automated file-size test fails `cargo test` above the ceiling | `main.rs::test_all_rust_files_under_reasonable_limit` | ✅ Active (600-line pragmatic gate; 180-line target v0.1.0) |
| `nemo-mbridge-resiliency` | Resource Hog Detector + Spike Blackbox | `process_model.rs`, `table_body.rs`, `table_widgets.rs` | ✅ Active (≥15% RAM; absolute ≥1.5 GiB planned) |
| `tao-launch-workflow` | Pre-launch Safety Gate for PID | `collector.rs` (per-process action point) | 🛡️ Design contract (v0.1.0) |
| `jetson-memory-audit` | Before/After Memory Reclamation Audit | `collector.rs` (`own_working_set_mb`, `trim_own_working_set`) | 🔬 Measurement primitives ready; audit record v0.1.0 |
| `nemo-mbridge-perf-moe-optimization-workflow` | Three Walls Bottleneck Radar (`SYSTEM OPTIMAL` / `MEMORY WALL` / `COMPUTE WALL` / `I/O WALL`) | Memory module now; CPU v0.2.0; Storage/Network v0.3.0–v0.4.0 | 🧭 Partial (`MEMORY WALL` only) |
| `medtech-model-evidence-export` | Sanitized Diagnostic Report with `[REDACTED]` masking | Reporting/export layer | 📋 Planned (v1.0.0) |

---

**Document Version:** `v0.0.9` (synced with `Cargo.toml`)
**Last Updated:** 2026-11-02
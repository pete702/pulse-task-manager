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
sysinfo = { version = "=0.30.12", default-features = false }
windows-sys = { version = "0.52.0", features = ["Win32_System_ProcessStatus", "Win32_System_Threading", "Win32_Foundation", "Win32_Globalization"] }
```

| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` | 0.27.2 | Native egui framework (window, event loop, wgpu) |
| `egui` | 0.27.2 | Immediate-mode GUI |
| `egui_plot` | 0.27.2 | Real-time graphs (CPU/RAM/GPU load) |
| `egui_extras` | 0.27.2 | Table, Strip, Image, RetainedImage |
| `sysinfo` | 0.30.12 | Cross-platform system info (CPU, RAM, Disks, Processes) — **no rayon thread pool** |
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
| **4. Universal UI Perfection & Zero-Overhead Engine** | **v0.0.10–v0.0.11** | Core/UI decomposition (50 modular files), panel order fix (top→bottom→left), rich telemetry sidebar mini-cards, kernel tiles 66px fixed height, `▲ HOG` Frame badge, CFG virtual memory ≥1 TiB normalization, adaptive chart height, tooltip formatter, trim-once startup, OS jitter probe, heuristic mode badge (GAMING/HEAVY/WORKSTATION/IDLE), minimized zero-GPU hibernation, global hotkeys (Ctrl+F, Esc), 2-thread architecture (no rayon), all files ≤180 lines | ✅ **Done** (v0.0.11) |
| **5. CPU Module** | v0.1.0–v0.2.0 | Per-core usage, frequency, temperature (MSR), `egui_plot` graphs, top consumers table | ⏳ Planned |
| **6. Storage I/O Module** | v0.2.0–v0.3.0 | Disk read/write throughput, latency, queue depth, SMART health, partition usage | ⏳ Planned |
| **7. Network & GPU Module** | v0.3.0–v0.5.0 | Per-interface throughput, TCP/UDP stats, gamer ping, DXGI GPU usage/VRAM/temp | ⏳ Planned |
| **8. Mini HUD Overlay** | v0.5.0–v1.0.0 | Always-on-top overlay, hotkey toggle, customizable panels, export to HTML/JSON | 🎯 **Final Target** |

---

## 7. Project Structure (Reference Tree — 50 Modular Files)

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
│   ├── main.rs                      # Entry point: eframe::run_native, --self-test, icon, windows_subsystem
│   ├── app.rs                       # Root App struct, state, Memory sub-view, update loop (≤153 lines)
│   ├── app_tabs.rs                  # ActiveTab enum + global hotkeys (Ctrl+F, Esc) (73 lines)
│   ├── theme.rs                     # Obsidian HUD tokens + style builder + apply_obsidian_theme
│   ├── theme_fonts.rs               # System fonts loader (Segoe UI / Consolas from Windows Fonts)
│   ├── i18n/                        # Internationalization package
│   │   ├── mod.rs                   # Core: Lang, Key, current_lang, set_lang, init_language, detect_os_language (126 lines)
│   │   ├── keys.rs                  # Key enum (all translation keys)
│   │   ├── frame.rs                 # Window chrome translations (top bar, sidebar, status bar, self-test)
│   │   ├── memory.rs                # Memory & Process module translations (cards, tiles, chart, table)
│   │   └── tests/                   # i18n unit tests (basic, translations UK/EN, formatted strings)
│   ├── core/
│   │   ├── mod.rs                   # SystemSnapshot (uptime, pulse_ram_mb, memory, os_scheduler_jitter_ms), mpsc channel types, spawn_telemetry_worker
│   │   ├── worker.rs                # Background thread: 1000 ms interval, process scan + trim-once + self RSS + os_scheduler_jitter_ms → channel (156 lines)
│   │   ├── selftest.rs              # Headless self-test + CI/GUI dispatch
│   │   ├── selftest_app.rs          # GUI SelfTestApp for --self-test live mode
│   │   └── selftest_tests.rs        # Architectural tests: file size ≤180, zero Cyrillic/Uk in .rs files (137 lines)
│   ├── ui/
│   │   ├── mod.rs                   # UI chrome + sidebar + placeholder tab exports
│   │   ├── chrome.rs                # Top panel rendering only (80 lines)
│   │   ├── sidebar.rs               # Rich telemetry sidebar with mini-cards (174 lines)
│   │   ├── status_bar.rs            # Bottom status bar: heuristic mode badge + OS jitter
│   │   └── placeholder_tab.rs       # Placeholder card for unimplemented tabs (Cpu, Storage, Network, System)
│   ├── modules/
│   │   ├── mod.rs                   # Pub re-exports all modules
│   │   └── memory/
│   │       ├── mod.rs               # Module exports
│   │       ├── model.rs             # MemorySnapshot, MemoryHistory (900 pts / 15 min), ChartWindow (146 lines)
│   │       ├── process_model.rs     # ProcessMemoryItem, clone grouping, sorting, MemorySpikeEvent (145 lines)
│   │       ├── collector.rs         # sysinfo + windows-sys (K32GetPerformanceInfo, K32GetProcessMemoryInfo, K32EmptyWorkingSet); CFG virtual memory ≥1 TiB normalization
│   │       ├── ram_card.rs          # RAM card rendering with traffic-light progress bar
│   │       ├── swap_commit_card.rs  # Swap & Commit Charge cards
│   │       ├── kernel_tiles.rs      # 4 kernel tiles with render_tile helper, 66px fixed height, integer MiB (154 lines)
│   │       ├── win32_api.rs         # Win32 wrappers: own_working_set_mb, trim_own_working_set, K32GetPerformanceInfo
│   │       ├── history.rs           # MemoryHistory ring buffer, windowed points
│   │       ├── chart_view.rs        # Multi-horizon (1m/5m/15m) dual-line neon chart, adaptive height clamp(115,260), tooltip formatter (163 lines)
│   │       ├── page.rs              # Memory sub-view switcher (Overview / Processes) + compact RAM bar
│   │       ├── table_view.rs        # MemoryTableState + search/group/sort orchestration
│   │       ├── table_body.rs        # Virtualized TableBuilder rows + sortable header, ▲ HOG Frame badge (179 lines)
│   │       └── table_widgets.rs     # Spike banner, toolbar, [Nx] badges, byte formatting
└── tests/                           # Integration tests (optional)
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

## 10. Universal UI Perfection & Zero-Overhead Engine (v0.0.11)

### 10.1 Panel Registration Order (Fixed Bottom Bar Clipping)

```
1. egui::TopBottomPanel::top("top_header")        // 100% window width
2. egui::TopBottomPanel::bottom("bottom_status_bar")  // 100% window width — never clipped!
3. egui::SidePanel::left("left_sidebar")          // Sits cleanly between top and bottom
4. egui::CentralPanel::default()                  // Remaining space
```

### 10.2 Rich Telemetry Sidebar (`ui/sidebar.rs`)

- **Active tab card:** background `#161F2E`, border `1.0px #00F0FF`, rounded `6.0`
- **Inactive tab card:** background `#12141C` (hover `#181C26`), border `1.0px #1F2330`, rounded `6.0`
- **Memory (RAM) card content:**
  - Row 1: Title "◈ Memory (RAM)" + live "XX.X%" (traffic-light colored)
  - Row 2: Sleek 3.0px mini progress bar showing RAM usage %
  - Row 3: Muted monospace subtitle "X.XX / Y.YY GiB • Commit ZZ%"
- **Other 4 cards:** Title + muted subtitle ("Telemetry: Phase 2", "Telemetry: Phase 3")
- Rendered via `ui.interact(..., egui::Sense::click())` — no ugly button fill, no phantom focus ring

### 10.3 Kernel Tiles (`modules/memory/kernel_tiles.rs`)

- Single `render_tile` helper → all 4 tiles share one layout block
- Strict identical dimensions: `ui.set_min_size(vec2(width, 66.0)); ui.set_max_size(vec2(width, 66.0));`
- Kernel Pool subtitle: integer MiB formatting → `format!("Paged: {:.0} MiB • Non-Paged: {:.0} MiB", ...)` — never wraps

### 10.4 Process Table Fixes

- **`▲ HOG` badge:** Geometric `▲ HOG` inside `egui::Frame::none().fill(Color32::from_rgb(58,16,29)).stroke(Stroke::new(1.0, CRIMSON)).rounding(3.0).inner_margin(Margin::symmetric(5.0, 1.0))` — aligns cleanly with name and `[Nx]` badge
- **CFG Virtual Memory Anomaly:** On 64-bit Windows 11, CFG reserves 2 TiB per system process. In `collector.rs`: if `virt_bytes >= (1_u64 << 40)` (≥1 TiB), normalize that process's virtual memory to its actual working set (`ram`), so aggregated groups show realistic committed/virtual memory instead of 100+ TiB

### 10.5 Chart View (`modules/memory/chart_view.rs`)

- **Adaptive height:** `let chart_height = (ui.available_height() - 18.0).clamp(115.0, 260.0);` — X-axis labels always visible on resize
- **Tooltip formatter:** `.label_formatter(|name, value| { let secs = value.x.abs().round() as u32; if name.is_empty() { format!("{secs}s ago: {:.1}%", value.y) } else { format!("{name}\n{secs}s ago: {:.1}%", value.y) } })`

### 10.6 Zero-Overhead Engine (`core/worker.rs`, `app.rs`, `ui/status_bar.rs`)

| Optimization | Implementation |
|--------------|----------------|
| **Trim once at startup** | `trim_own_working_set()` called **only once** before the worker loop (not periodically) — eliminates page-fault thrashing while GUI is open |
| **OS Scheduler Jitter Probe** | `let os_scheduler_jitter_ms = sleep_start.elapsed().as_millis().saturating_sub(1000) as u32;` stored in `SystemSnapshot`, displayed in status bar |
| **Heuristic Workload Mode** | Computed from snapshot: any process group ≥2.5 GiB → `[ ◈ MODE: GAMING / HEAVY APP ]` (red); total RAM ≥75% → `[ ▣ MODE: WORKSTATION LOAD ]` (amber); else `[ ● MODE: DESKTOP IDLE ]` (cyan) |
| **Minimized Zero-GPU Hibernation** | In `app.rs` `update()`: if `ctx.input(|i| i.viewport().minimized.unwrap_or(false))` → `request_repaint_after(1000ms)` + early return (0% GPU/CPU while minimized, worker keeps recording) |
| **Global Hotkeys** | `Ctrl+F` → focuses Memory→Processes + clears search; `Esc` → clears search query |

### 10.7 Two-Thread Architecture (No Rayon)

- **Thread 1 (GUI):** `eframe` event loop, rendering, input handling
- **Thread 2 (Worker):** 1000 ms interval, `sysinfo` memory/process collection, Win32 kernel metrics, own RSS measurement, jitter measurement, sends `SystemSnapshot` via `mpsc`
- `sysinfo` with `default-features = false` → **no rayon thread pool** spawned anywhere
- Binary size: **4.38 MiB** (thin LTO, strip=symbols, codegen-units=1, panic=abort)

---

## 11. NVIDIA Agent Skills Engineering Patterns

| NVIDIA Agent Skill | PULSE Engineering Pattern | Implementation | Status |
|--------------------|---------------------------|----------------|--------|
| `ethos-orchestrate` / `mcore-testing` | **Self-Enforcing Architecture Guard** — automated test scans every `src/**/*.rs` file and fails `cargo test` when a file exceeds the line ceiling | `src/core/selftest_tests.rs::test_all_rust_files_under_reasonable_limit` (180-line hard limit) | ✅ Active |
| `nemo-mbridge-resiliency` | **Resource Hog Detector (`▲ HOG`) + Spike Blackbox** — flags processes at ≥15% of total RAM and records a `MemorySpikeEvent` when system RAM reaches ≥85%, including culprit PID and byte count | `process_model.rs`, `table_body.rs`, `table_widgets.rs` | ✅ Active (relative threshold; absolute ≥1.5 GiB check queued for v0.1.0) |
| `tao-launch-workflow` | **Pre-launch Safety Gate for PID** — before any Win32 action, reject PID 0 (System Idle), PID 4 (System), PULSE's own PID, and critical OS processes | Integration point reserved in `collector.rs`; per-process reclamation lands in v0.1.0 | 🛡️ Design contract |
| `jetson-memory-audit` | **Before/After Memory Reclamation Audit** — measure working-set bytes immediately before and after `K32EmptyWorkingSet`, then report the exact freed delta | `K32GetProcessMemoryInfo` / `own_working_set_mb` measurement primitives in `win32_api.rs`; audit record in v0.1.0 | 🔬 Primitive ready |
| `nemo-mbridge-perf-moe-optimization-workflow` | **Three Walls Bottleneck Radar** — classify the system as `SYSTEM OPTIMAL`, `MEMORY WALL`, `COMPUTE WALL`, or `I/O WALL` from live telemetry | `MEMORY WALL` available from RAM/commit usage today; `COMPUTE WALL` arrives with CPU module (v0.2.0), `I/O WALL` with Storage/Network (v0.3.0–v0.4.0) | 🧭 Partial |
| `medtech-model-evidence-export` | **Sanitized Diagnostic Report** — export a shareable diagnostic report with `[REDACTED]` masking for machine names, user paths, IP/MAC addresses, and other PII | Export layer planned for the v1.0.0 reporting milestone | 📋 Planned |

---

## 12. Working Set Monitoring & Safety-Gated Reclamation

### 12.1 Data Carried by the Snapshot

`MemorySnapshot` now carries the process table and the latest spike:

- `processes: Vec<RawProcessRow>` — `(PID, name, RAM bytes, virtual bytes)`.
- `latest_spike: Option<MemorySpikeEvent>` — the most recent level-triggered spike (RAM ≥ 85%), with timestamp, peak %, culprit name/PID/bytes.

`collect_process_memory()` refreshes with `ProcessRefreshKind::new().with_memory()` and never performs a full system scan; the worker attaches the result to `memory.processes` before sending the snapshot.

### 12.2 Own Working Set (`K32GetProcessMemoryInfo` + `K32EmptyWorkingSet`)

PULSE measures and trims **its own** process only:

| Step | API | Notes |
|------|-----|-------|
| Trim | `K32EmptyWorkingSet(GetCurrentProcess())` | Releases font/OpenGL buffers back to the OS. Called by the worker **once at startup** (not periodically) and by the app after OpenGL startup (frame 3) and every 30 frames. |
| Measure | `K32GetProcessMemoryInfo(GetCurrentProcess(), …)` → `WorkingSetSize` | `own_working_set_mb()` returns MiB; used for the self-diagnosis badge. |

All Win32 calls are narrow wrappers in `win32_api.rs`; `main.rs` keeps `#![deny(unsafe_code)]` and the wrappers carry a local `#[allow(unsafe_code)]` with a documented SAFETY comment.

### 12.3 Safety Gate for Future Per-Process Actions

Before PULSE ever calls `OpenProcess` / `K32EmptyWorkingSet` on **another** process (v0.1.0), the action must pass this gate:

1. Reject PID `0` (`System Idle Process`) and PID `4` (`System`).
2. Reject PULSE's own PID.
3. Reject critical OS processes (`smss.exe`, `csrss.exe`, `wininit.exe`, `services.exe`, `lsass.exe`, `winlogon.exe`).
4. Require an explicit row selection and a confirmation step before the Win32 call.

### 12.4 Reclamation Audit Contract

Every reclamation action must record `(pid, before_bytes, after_bytes, freed_bytes)` by reading the working set immediately before and after the trim, so the UI can report the exact reclaimed amount instead of an estimate.

---

## 13. System Jitter Probe & Heuristic Mode (v0.0.11)

### 13.1 OS Scheduler Jitter

- Measured in `core/worker.rs` at each 1000 ms sleep cycle
- `os_scheduler_jitter_ms = sleep_start.elapsed().as_millis().saturating_sub(1000) as u32`
- Stored in `SystemSnapshot.os_scheduler_jitter_ms`
- Displayed in status bar as "OS Jitter: X ms"

### 13.2 Universal Heuristic Workload Mode

Computed in `ui/status_bar.rs` from the latest `SystemSnapshot`:

| Condition | Badge | Color |
|-----------|-------|-------|
| Any single process group ≥ 2.5 GiB RAM | `[ ◈ MODE: GAMING / HEAVY APP ]` | Red (`ACCENT_RED`) |
| Total RAM usage ≥ 75% | `[ ▣ MODE: WORKSTATION LOAD ]` | Amber (`ACCENT_AMBER`) |
| Otherwise | `[ ● MODE: DESKTOP IDLE ]` | Cyan (`ACCENT_CYAN`) |

---

**Document Version:** `v0.0.11` (synced with `Cargo.toml`)
**Last Updated:** 2026-11-03
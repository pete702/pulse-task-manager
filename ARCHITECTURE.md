# ARCHITECTURE.md — PULSE Project Immutable Laws

> This file defines the **constitution** of the codebase. Any deviation requires an explicit architect decision and an update to this document.

---

## 1. Modularity (Module Boundaries)

| Rule | Value |
|------|-------|
| **Max lines per file** | **150** (hard limit) |
| **Module structure** | `src/modules/<name>/` with mandatory split: |
| | • `model.rs` — data, pure logic, **unit tests** (`#[cfg(test)]`) |
| | • `collector.rs` — OS data collection (Windows API, `sysinfo`) |
| | • `view.rs` — UI (egui/eframe) |
| **Forbidden** | Mixing collection logic with UI, moving business logic into `view.rs` |
| **Public module API** | Exported **only** via module's `mod.rs` (`pub use model::*; pub use collector::*; pub use view::*;`) |

> **Why 150 lines?** Guarantees: 30-second readability, easy code review, isolated unit tests, no "god files".

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
windows-sys = { version = "0.52.0", features = ["Win32_System_ProcessStatus", "Win32_System_Threading", "Win32_Foundation"] }
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
| **3** | **Unit Tests** | `cargo test` | **All tests pass** |
| **4** | **Self-Test** | `cargo run -- --self-test` | Prints `SELF-TEST PASSED` and exits 0 |
| **5** | **Build & Export** | `cargo build --release` → copy to `../PULSE_BUILDS/pulse_vX.Y.Z.exe` + `PULSE_LATEST.exe` | Both `.exe` files exist and run |
| **6** | **Clean Backup** | Copy `src/`, `Cargo.toml`, `Cargo.lock`, `*.md` to `../PULSE_BACKUPS/vX.Y.Z_verified/` (NO `target/`, NO `.git/`) | Backup folder complete |
| **7** | **Documentation** | Update `CHANGELOG.md` (EN), append to `../PULSE_AI_CONTEXT.md` (UA), update `../PULSE_UA_MANUAL.md` (UA) — **never delete history** | All three files current |
| **8** | **Git Push** | `git add . && git commit -m "vX.Y.Z: description" && git push` | Remote `main` updated |

---

## 6. Project Structure (Reference Tree)

```
pulse_manager/
├── .gitignore
├── README.md
├── ARCHITECTURE.md
├── CHANGELOG.md
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs                 # Entry point: eframe::run_native
│   ├── app.rs                  # Root App struct, state, update loop
│   ├── theme.rs                # Obsidian HUD tokens + style builder + fonts
│   ├── i18n.rs                 # Language switcher (UA/EN)
│   ├── core/
│   │   ├── mod.rs              # SystemSnapshot, mpsc channel types, spawn_telemetry_worker
│   │   └── worker.rs           # Background thread: 500ms interval, System::uptime() → channel
│   ├── modules/
│   │   ├── mod.rs              # Pub re-exports all modules
│   │   ├── memory/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs        # PhysicalMemory, SwapMemory, CacheCleaner
│   │   │   ├── collector.rs    # sysinfo + windows-sys (EmptyWorkingSet)
│   │   │   └── view.rs         # RAM/Swap cards, cleanup button, graphs
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

## 7. Forbidden Patterns (Anti-Patterns)

| Pattern | Why Forbidden | Alternative |
|---------|---------------|-------------|
| `unwrap()` / `expect()` in production code | Panic = UI crash | `Result` + `?` / `tracing::error!` + fallback |
| Global `static mut` / `lazy_static` | Unsafe, hard to test | Dependency Injection via `App` state |
| Blocking calls in UI thread | Frame freeze (jank) | `tokio::task::spawn_blocking` / rayon / channels |
| Hardcoded colors/sizes in `view.rs` | Theme inconsistency | `theme.rs` tokens + `Theme::apply(&mut ctx)` |
| Module without `model.rs`/`collector.rs`/`view.rs` | Violates separation of concerns | Create three files or merge with another module |

---

## 8. CI/CD Gates (Pre-Push Checks)

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

**Document Version:** `v0.0.3` (synced with `Cargo.toml`)
**Last Updated:** 2026-11-02
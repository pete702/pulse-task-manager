# Contributing to PULSE

## Pinned 6-Crate Stack (MSRV 1.78)

| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` | `=0.27.2` | GUI framework |
| `egui` | `=0.27.2` | Immediate-mode UI |
| `egui_plot` | `=0.27.2` | Real-time charts |
| `egui_extras` | `=0.27.2` | TableBuilder, extras |
| `sysinfo` | `=0.30.12` | System/process metrics (default-features=false) |
| `windows-sys` | `=0.52.0` | Win32 FFI (ProcessStatus, Threading, Foundation, Globalization, Security) |

**No other dependencies are accepted.** Version pins are exact (`=`). MSRV is **1.78.0** — do not use features stabilized after 1.78 (e.g., `LazyLock`, `is_multiple_of`).

## Code Structure & Hygiene

- **Line cap**: Every `.rs` file in `src/` and `tests/` must be `<= 180` lines after `cargo fmt`. Enforced by `src/core/selftest_tests.rs` and `tests/docs_guard.rs`.
- **Zero `rustfmt::skip`**: The attribute is forbidden repository-wide. Run `cargo fmt --all` before commit.
- **Language hygiene**: 0 Cyrillic characters (`U+0400..=U+052F`), 0 NUL bytes (`\0`) in all repository files. Enforced by `tests/docs_guard.rs`.
- **Module boundaries**: Submodule tests live in sibling `*_tests.rs` files inside `src/`.

## Mandatory Checks (CI Parity)

Before pushing, run locally and ensure all pass:

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
.\target\release\pulse_manager.exe --self-test
```

The `--self-test` must print `SELF-TEST PASSED` and exit 0.

## License Agreement

By contributing, you agree that your contributions will be licensed under the **MIT License** (same as the project). See `LICENSE` for full text.
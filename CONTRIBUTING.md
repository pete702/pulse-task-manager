# Contributing to PULSE

Thank you for your interest in contributing to PULSE! This document outlines the
process and standards for contributions.

## Code of Conduct

All contributors are expected to maintain a respectful and professional environment.
Harassment, discrimination, or disruptive behavior will not be tolerated.

## Developer Certificate of Origin (DCO)

By contributing to this project, you certify that:

> **Signed-off-by: Your Name <your.email@example.com>**
>
> You certify that you have the right to submit the contribution under the
> project's license (MIT), and that the contribution is your original work
> or based on work that has been appropriately licensed.

Include `Signed-off-by` in every commit message:
```bash
git commit -s -m "Fix issue: description"
```

## Pull Request Process

1. Fork the repository and create a feature branch from `main`.
2. Ensure all code follows the project style (`cargo fmt --all`).
3. Run the full verification chain before submitting:
   ```powershell
   cargo fmt --all
   cargo clippy --locked --all-targets -- -D warnings
   cargo test --locked -q
   cargo build --release --locked
   ```
4. Keep changes focused and minimal. Large refactors should be discussed first.
5. Update `CHANGELOG.md` with a brief entry under `## [Unreleased]`.
6. Submit the PR with a clear title and description.

## Coding Standards

- **Rust Edition**: 2021
- **MSRV**: 1.85 (stable toolchain only)
- **Line Limit**: Every `.rs` file ≤ 280 lines after `cargo fmt --all`
- **Dependencies**: 6 direct runtime crates + 1 MSRV pin, all pinned exactly in `Cargo.toml`
- **No Unstable Features**: No `#![feature(...)]`, no nightly-only syntax
- **Zero Allocations on Hot Paths**: Prefer stack allocation, `Arc<str>` for shared strings
- **Forbidden**: `rustfmt::skip`, Cyrillic characters, NUL bytes, `unwrap()`/`expect()` on hot paths

## Architecture Guidelines

- **Zero-Copy Telemetry**: Background worker sends `Arc<SystemSnapshot>` via bounded channel
- **O(1) UI Caching**: `MemoryTableState` keyed by `process_scan_id` short-circuits unchanged scans
- **Adaptive Cadence**: 1 Hz system telemetry, 3 s process scan (Processes tab), 5 s (Overview), 0 fps minimized
- **PID Safety**: All mutating actions verify 3-point identity (basename + creation_time + full_path)

## Testing

- Unit tests in `#[cfg(test)]` modules alongside source
- Integration tests in `tests/`
- `docs_guard` tests enforce line limits, forbidden tokens, and README link freshness
- All 159 automated checks (155 unit + 4 guard) must pass on Windows x64 MSVC

## Legal

All contributions are licensed under the MIT License (see `LICENSE`).

By submitting a contribution, you grant **Petro (pete702)** perpetual rights to:
- Distribute the contribution under the MIT License (open-source)
- Include the contribution in official commercial/enterprise distributions of PULSE
- Sublicense the contribution as part of PULSE binary releases

This grant is irrevocable and applies to all current and future versions of PULSE.

## Reporting Issues

- Use GitHub Issues for bugs and feature requests
- Include: OS version, PULSE version, steps to reproduce, expected vs actual behavior
- Attach relevant logs or screenshots if applicable

## Questions?

Open a GitHub Discussion or issue for any questions about contributing.
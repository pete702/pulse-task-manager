# Changelog
All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.
## [0.4.160] - 2026-04-12
- **cleanup**: removed duplicate tests and repo walk checks.
## [0.4.159] - 2026-04-12
- **storage/ui**: low-space alert summary, volume labels, I/O tooltip.
## [0.4.158] - 2026-04-12
- **storage**: disk refresh cadence, IO spike threshold, handle TTL, IO attribution.
## [0.4.157] - 2026-04-12
- **network**: probe guard/watchdog, adapter prune, traffic threshold, card epsilon.
## [0.4.156] - 2026-04-12
- **memory delta/top cards**: probe budget, short-interval projection, cached tooltip.
## [0.4.155] - 2026-04-12
- **memory spike**: fallback by abs delta, none on zero.
## [0.4.154] - 2026-04-12
- **cpu table**: group count label and delta tooltip helper.
## [0.4.153] - 2026-04-12
- **cpu table menu**: require resolved exe path for group actions.
## [0.4.152] - 2026-04-12
- **cpu table/top cards**: quantized CPU sort and precision bar color helper.
## [0.4.151] - 2026-04-12
- **ui chrome/render**: removed header glyph and fixed scanline placement.
## [0.4.150] - 2026-04-12
- **i18n/chrome**: fixed diamond glyph escape regression.
## [0.4.149] - 2026-04-12
- **cpu collector/model**: probe budget, short-dt disk rates, single-core hog helper, format epsilon.
## [0.4.148] - 2026-04-12
- **ring history/main**: binary-search window bounds and console attach binding.
## [0.4.147] - 2026-04-12
- **worker**: panic recovery, drop counter, scheduler overshoot helper.
## [0.4.146] - 2026-04-12
- **ui tables/sidebar**: theme usage bars and FIFO badge cache.
## [0.4.145] - 2026-04-12
- **ui chrome**: header glyph, scanline spacing, and user-mode telemetry label.
## [0.4.144] - 2026-04-12
- **theme**: disabled label selection and bounded font reads.
## [0.4.143] - 2026-04-12
- **handle**: image path buffer resize and critical process error diagnostics.
## [0.4.142] - 2026-04-12
- **sys metrics**: DWM dark titlebar HRESULT handling and latch.
## [0.4.141] - 2026-04-12
- **platform trim**: Win32 error mapping and exited-process handling fixes.
## [0.4.140] - 2026-04-12
- **app tests**: fixed stale self-trim assertion for 5s startup delay.
## [0.4.139] - 2026-04-12
- **app trim**: manual Trim Memory invokes self-trim directly; removed early frame-count shortcut so startup trim waits 5s.
## [0.4.138] - 2026-04-12
- **app_cmd**: real command dispatch; UI emit for tab/pause/trim.
## [0.4.137] - 2026-04-12
- **cpu actions/cols**: kill post-verify; condensed cols; changelog fix.
## [0.4.136] - 2026-04-12
- **table cols/menu**: post-action verify; condensed cols.
## [0.4.135] - 2026-04-12
- **app_cmd**: AppCommand enum; ARCHITECTURE 5-crate core.
## [0.4.134] - 2026-04-12
- **group actions**: leader-only identity verify; track all members.
## [0.4.133] - 2026-04-12
- **table cols**: residual correction, narrow separators, conditional store.
## [0.4.132] - 2026-04-12
- **table_kit + deps**: shared table helpers; removed unused egui_extras.
## [0.4.131] - 2026-04-12
- **i18n**: FromStr for Lang; removed obsolete memory-table keys (B19).
## [0.4.130] - 2026-04-12
- **Status bar**: documented cache quantization and CPU alert tier (B19-B-01, B19-B-03).
## [0.4.129] - 2026-04-12
- **CPU**: table columns residual correction (B22), table actions identity (B23), README v0.4.129.
## [0.4.128] - 2026-04-12
- **Storage model**: explicit per-volume read/write summation (B14-B-05).
- **Storage inspector**: HashSet drive filter in group() (B15-B-04).
## [0.4.127] - 2026-04-12
- **UI primitives**: tactical ticks always render (B18-B-02).
- **Sidebar**: Enter activates, request_focus on nav; scope-keyed alert latch (B18-B-03, B18-B-04).
- **Three Walls**: test for both cpu and memory None (B18-B-08).
- **Util/theme fonts**: UTC doc note, MAX_FONT_BYTES constant (B20-B-03, B20-B-04).
## [0.4.126] - 2026-04-12
- **Network chart tests**: restored traffic-window axis test (B16 WHEEL 5).
- **Latency probe tests**: stabilized jitter assertion (B16 WHEEL 6).
- **Network cards**: pinned bandwidth tier and cached KPI strings (B17-B-03).

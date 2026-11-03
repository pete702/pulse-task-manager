# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Current release |

Only the latest minor release in the `0.1.x` series receives security updates.

## Local-Only Telemetry Guarantee

PULSE is a **local-only** Windows telemetry HUD. It performs **zero network connections** of any kind:

- No telemetry, metrics, or usage data is transmitted externally.
- No update checks, no phoning home, no crash reporting to remote endpoints.
- All Win32 API calls target the local machine only (`K32GetPerformanceInfo`, `K32GetProcessMemoryInfo`, `K32EmptyWorkingSet`, `OpenProcess`, `TerminateProcess`, `QueryFullProcessImageNameW`, `IsProcessCritical`, `OpenProcessToken`, `GetUserDefaultUILanguage`).
- The binary is a single portable `.exe` with no background service, no installer, and no scheduled tasks.

## PID Safety Gate Boundaries

Before any mutating action (working-set trim, process termination), PULSE enforces a **PID Safety Gate** (`src/modules/memory/safe_handle.rs`):

- **Always blocked**: PID 0 (System Idle Process), PID 4 (System), current PULSE process PID.
- **Critical OS processes** (27): `System`, `Registry`, `Memory Compression`, `smss.exe`, `csrss.exe`, `wininit.exe`, `services.exe`, `lsass.exe`, `lsaiso.exe`, `svchost.exe`, `dwm.exe`, `winlogon.exe`, `fontdrvhost.exe`, `sihost.exe`, `taskhostw.exe`, `msmpeng.exe`, `nissrv.exe`, `securityhealthservice.exe`, `spoolsv.exe`, `audiodg.exe`, `wmiprvse.exe`, `searchindexer.exe`, `startmenuexperiencehost.exe`, `searchhost.exe`, `shellexperiencehost.exe`, `textinputhost.exe`, `conhost.exe`.
- **Known anti-cheat processes** (11): `vgc.exe`, `vgtray.exe`, `easyanticheat.exe`, `easyanticheat_eos.exe`, `beservice.exe`, `bedaisy.sys`, `zksvc.exe`, `faceitclient.exe`, `eseaclient.exe`, `ricochet.exe`, `hoyokprotect.sys`.

The gate uses case-insensitive matching with `.exe` stripping. Mutating paths fail closed: `is_handle_critical` treats any `IsProcessCritical` failure as critical, and `verify_handle_identity` returns `false` on any `QueryFullProcessImageNameW` failure, empty path, or name mismatch. `OpenProcessToken` + `GetTokenInformation` provide the elevation check.

## Vulnerability Reporting

Report security vulnerabilities via **GitHub Security Advisories** (private disclosure):

1. Go to the repository Security tab → "Report a vulnerability".
2. Provide a clear description, reproduction steps, and impact assessment.
3. Do not open public issues for security findings.

We aim to acknowledge within 72 hours and provide a mitigation timeline.
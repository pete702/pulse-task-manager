# ⚡ PULSE // Next-Gen System Telemetry for Windows

> **Надшвидкий модульний диспетчер задач на Rust + egui у стилі "Obsidian HUD"**

---

## 🇺🇦 Українська

### Опис
**PULSE** — це сучасний системний монітор для Windows, написаний на Rust з використанням `eframe`/`egui`. Проект побудований на принципах **суворої модульності**, **нульових витрат на абстракції** та **еталонного UI** у стилі "Obsidian HUD".

### Ключові можливості (план)
- 📊 **Детальний моніторинг фізичної RAM та Swap (файл підкачки)**
- ⚡ **Очищення кешу ОЗП в 1 клік** (Working Set Trim / EmptyWorkingSet)
- 🗂 **Групування процесів** за назвою, користувачем, ресурсами
- 🎨 **Obsidian HUD UI**: темний фон `#0B0C10`, картки `#12141C`, рамки `#1F2330`, заокруглення `6.0`
- 🔢 **Усі цифри (МБ, ГБ, %, PID) — виключно моноширинний шрифт** (`.monospace()`)
- 🚦 **Світлофор навантаження**: `<60%` — `#00F0FF` (ціан), `60–85%` — `#FFB800` (бурштиновий), `≥85%` — `#FF2A54` (червоний неон)

### Архітектура
- **Модульність**: жоден файл ≤ 150 рядків. Кожен модуль у `src/modules/<назва>/` поділений на:
  - `model.rs` — дані + чиста логіка + unit-тести
  - `collector.rs` — збір даних з ОС (Windows API через `windows-sys`, `sysinfo`)
  - `view.rs` — UI (egui)
- **Документація**: всі публічні API — українською (`///`)
- **Фіксовані версії залежностей** — 100% сумісність API

### Версія
**Поточна: `v0.0.1`** — ініціалізація проекту, структура, CI-готовий каркас.

### Запуск
```bash
cargo run --release
```

### Вимоги
- Rust 1.78+ (MSVC toolchain для Windows)
- Windows 10/11 (x64)

---

## 🇬🇧 English

### Description
**PULSE** is a modern system monitor for Windows, written in Rust using `eframe`/`egui`. Built on **strict modularity**, **zero-cost abstractions**, and a **reference-grade "Obsidian HUD" UI**.

### Key Features (Planned)
- 📊 **Detailed physical RAM & Swap (pagefile) monitoring**
- ⚡ **One-click RAM cache cleanup** (Working Set Trim / EmptyWorkingSet)
- 🗂 **Process grouping** by name, user, resource usage
- 🎨 **Obsidian HUD UI**: dark bg `#0B0C10`, cards `#12141C`, borders `#1F2330`, radius `6.0`
- 🔢 **All numbers (MB, GB, %, PID) — monospace only** (`.monospace()`)
- 🚦 **Traffic-light load colors**: `<60%` — `#00F0FF` (cyan), `60–85%` — `#FFB800` (amber), `≥85%` — `#FF2A54` (neon red)

### Architecture
- **Modularity**: no file > 150 lines. Each module in `src/modules/<name>/` strictly split into:
  - `model.rs` — data + pure logic + unit tests
  - `collector.rs` — OS data collection (Windows API via `windows-sys`, `sysinfo`)
  - `view.rs` — UI (egui)
- **Documentation**: all public APIs documented in Ukrainian (`///`)
- **Pinned dependency versions** — 100% API compatibility guarantee

### Version
**Current: `v0.0.1`** — project initialization, structure, CI-ready scaffold.

### Run
```bash
cargo run --release
```

### Requirements
- Rust 1.78+ (MSVC toolchain for Windows)
- Windows 10/11 (x64)

---

## 📜 License
MIT License — see [LICENSE](LICENSE) for details.
# ARCHITECTURE.md — Незмінні Закони Проекту PULSE

> Цей файл визначає **конституцію** кодової бази. Будь-яке відхилення вимагає явного рішення архітектора та оновлення цього документа.

---

## 1. Модульність (Module Boundaries)

| Правило | Значення |
|---------|----------|
| **Макс. рядків у файлі** | **150** (жорсткий ліміт) |
| **Структура модуля** | `src/modules/<name>/` з обов'язковим подилом: |
| | • `model.rs` — дані, чиста логіка, **unit-тести** (`#[cfg(test)]`) |
| | • `collector.rs` — збір даних з ОС (Windows API, `sysinfo`) |
| | • `view.rs` — UI (egui/eframe) |
| **Заборона** | Мішати логіку збору даних з UI, виносити бізнес-логіку у `view.rs` |
| **Публічний API модуля** | Експортується **тільки** через `mod.rs` модуля (`pub use model::*; pub use collector::*; pub use view::*;`) |

> **Чому 150 рядків?** Це гарантує: читабельність за 30 сек, легкий code review, ізольовані юніт-тести, відсутність "god files".

---

## 2. Документація та Коментарі (Documentation Standard)

| Елемент | Вимога |
|---------|--------|
| **Публічні структури** | `///` українською — призначення, інваріанти, приклад використання |
| **Публічні поля** | `///` — семантика, одиниці виміру, допустимі діапазони |
| **Публічні функції/методи** | `///` — контракт: що робить, аргументи, повернення, помилки, side-effects |
| **Приватні елементи** | За бажанням, але **обов'язково** для складної логіки |
| **Мова** | **Тільки українська** для doc-коментарів (`///`). Кодові коментарі (`//`) — допускаються англійською. |

> **Приклад:**
> ```rust
> /// Физична пам'ять системи: загальний обсяг, доступний, використовуваний.
> /// Усі значення — у **байтах**.
> #[derive(Debug, Clone, Copy, PartialEq, Eq)]
> pub struct PhysicalMemory {
>     /// Загальний обсяг ОЗП (bytes).
>     pub total: u64,
>     /// Доступний для виділення без свопінгу (bytes).
>     pub available: u64,
>     /// Поточне використання (bytes).
>     pub used: u64,
> }
> ```

---

## 3. Фіксовані Версії Крейтів (Pinned Dependencies)

> **ЗАБОРОНЯЄТЬСЯ** використовувати `^`, `~`, `*` або `latest`. Версії **жорстко фіксовані** для 100% повторюваності збирання та сумісності API.

```toml
[dependencies]
eframe = "0.27.2"
egui = "0.27.2"
egui_plot = "0.27.2"
egui_extras = "0.27.2"
sysinfo = "0.30.12"
windows-sys = { version = "0.52.0", features = ["Win32_System_ProcessStatus", "Win32_System_Threading", "Win32_Foundation"] }
```

| Крейт | Версія | Призначення |
|-------|--------|-------------|
| `eframe` | 0.27.2 | Нативний фреймворк для egui (window, event loop, wgpu) |
| `egui` | 0.27.2 | Immediate-mode GUI |
| `egui_plot` | 0.27.2 | Графіки завантаження CPU/RAM/GPU |
| `egui_extras` | 0.27.2 | Table, Strip, Image, RetainedImage |
| `sysinfo` | 0.30.12 | Крос-платформний збір системної інфо (CPU, RAM, Disks, Processes) |
| `windows-sys` | 0.52.0 | Прямі Windows API (PSAPI, Threading, Foundation) для EmptyWorkingSet, точніших метрик |

> **Оновлення версій** — тільки через ARCHITECTURE.md + CHANGELOG.md + тестування `cargo check` / `cargo test`.

---

## 4. Дизайн-система "Obsidian HUD" (Design Tokens)

| Token | Значення | Застосування |
|-------|----------|--------------|
| `BG_WINDOW` | `#0B0C10` | Фон головного вікна (`CentralPanel`, `SidePanel`) |
| `BG_CARD` | `#12141C` | Фон карток, панелей процесів, графіків |
| `BORDER` | `#1F2330` | Рамки карток, роздільники таблиць (1px) |
| `RADIUS` | `6.0` | Заокруглення `CornerRadius::same(6.0)` |
| `MONO_FONT` | `FontId::monospace(size)` | **Усі цифри**: MB, GB, %, PID, Hz, °C — **тільки моноширинний** |

### Світлофор Навантаження (Traffic Light)

| Діапазон | Колор | Hex | Назва |
|----------|-------|-----|-------|
| `< 60%` | ![#00F0FF](https://via.placeholder.com/10/00F0FF/000000?text=+) | `#00F0FF` | **Ціан (Cyan)** — норма |
| `60–85%` | ![#FFB800](https://via.placeholder.com/10/FFB800/000000?text=+) | `#FFB800` | **Бурштиновий (Amber)** — попередження |
| `≥ 85%` | ![#FF2A54](https://via.placeholder.com/10/FF2A54/000000?text=+) | `#FF2A54` | **Червоний неон (Neon Red)** — критично |

> **Правило**: колір визначається **поточним відсотком** ресурсу (RAM, CPU, GPU, Disk). Застосовується до: тексту відсотка, наповнення прогрес-бару, точок на графіках.

---

## 5. Definition of Done (DoD) — Правило Завершення Кожного Кроку

> **Агент ЗОБОВ'ЯЗАНИЙ** виконати **5 дій** на завершення кожного кроку (версії):

### 5.1. Якість коду
- `cargo check` — **0 помилок компіляції**
- `cargo test` — **усі тести проходять** (unit + integration, якщо є)

### 5.2. Версіонування
- Оновити `version` у `Cargo.toml` (SemVer: `MAJOR.MINOR.PATCH`)

### 5.3. Офіційний CHANGELOG.md (всередині репозиторію)
- Додати запис про нову версію згідно [Keep a Changelog](https://keepachangelog.com/):
  - `## [vX.Y.Z] - YYYY-MM-DD`
  - `### Added / Changed / Fixed / Removed`

### 5.4. Зовнішня Інженерна Історія — `../PULSE_AI_CONTEXT.md`
> **СУВОРЕ ПРАВИЛО**: **Ніколи не стирати старі записи**. Паспорт проекту оновлюється зверху, хронологічна історія дописується знизу.

Структура запису версії:
1. **Які файли створено/змінено** і що саме реалізовано
2. **ЖУРНАЛ ПОМИЛОК ТА ВИПРАВЛЕНЬ (Bug & Fix Log)**: детально — помилки компіляції/тестів, причина, точний спосіб виправлення (якщо немає — `"Скомпільовано з першої спроби без помилок"`)
3. **План на наступну версію**

### 5.5. Git-коміт
```bash
git add .
git commit -m "vX.Y.Z: короткий опис змін"
```

---

## 6. Сруктура Проекту (Reference Tree)

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
│   ├── theme.rs                # Obsidian HUD tokens + style builder
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

## 7. Заборонені Патерни (Anti-Patterns)

| Патерн | Чому заборонено | Альтернатива |
|--------|-----------------|--------------|
| `unwrap()` / `expect()` у продакшн-коді | Паніка = краш UI | `Result` + `?` / `tracing::error!` + fallback |
| Глобальні `static mut` / `lazy_static` | Небезпечно, важко тестувати | Dependency Injection через `App` state |
| Блокуючі виклики в UI потоці | Заморожування кадру (jank) | `tokio::task::spawn_blocking` / rayon / channels |
| Хардкод кольорів/розмірів у `view.rs` | Несумісність з темою | `theme.rs` токени + `Theme::apply(&mut ctx)` |
| Модуль без `model.rs`/`collector.rs`/`view.rs` | Порушення ізоляції відповідальності | Створити три файли або об'єднати з іншим модулем |

---

## 8. CI/CD Gates (Перевірки перед Push)

```yaml
# .github/workflows/ci.yml (план)
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo check --all-targets
- cargo test --all-targets
- cargo build --release
```

> Усі гейти **обов'язкові**. Merge без зелених чеків — заборонено.

---

**Версія документа:** `v0.0.1` (синхронізовано з `Cargo.toml`)
**Останнє оновлення:** 2026-11-02
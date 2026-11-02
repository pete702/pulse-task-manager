//! Ядро телеметрії PULSE: фоновий робітник, снепшоти системи, канали зв'язку.

pub mod worker;

use std::sync::mpsc;

/// Снепшот стану системи, що передається з фонового потоку в UI.
/// Поки містить лише uptime; у v0.0.3+ розшириться метриками пам'яті/CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemSnapshot {
    /// Час роботи системи у секундах (від `sysinfo::System::uptime()`).
    pub uptime_secs: u64,
}

impl SystemSnapshot {
    /// Створює новий снепшот із заданого uptime.
    #[inline]
    pub fn new(uptime_secs: u64) -> Self {
        Self { uptime_secs }
    }
}

/// Тип каналу для передачі снепшотів: `Sender` у фоновому потоці, `Receiver` у UI.
pub type SnapshotTx = mpsc::Sender<SystemSnapshot>;
pub type SnapshotRx = mpsc::Receiver<SystemSnapshot>;

/// Запускає фоновий потік збору телеметрії.
/// Повертає `Receiver` для читання снепшотів у головному потоці.
pub fn spawn_telemetry_worker() -> SnapshotRx {
    worker::spawn()
}

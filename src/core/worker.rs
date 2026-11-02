//! Фоновий робітник телеметрії: періодично збирає uptime та надсилає через mpsc канал.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sysinfo::System;

use crate::core::{SnapshotTx, SystemSnapshot};

/// Інтервал оновлення телеметрії (500 мс).
const WORKER_INTERVAL: Duration = Duration::from_millis(500);

/// Запускає фоновий потік, який кожні 500 мс читає `System::uptime()`
/// і надсилає `SystemSnapshot` через канал.
/// Повертає `Receiver` для UI потоку.
pub fn spawn() -> mpsc::Receiver<SystemSnapshot> {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<SystemSnapshot>) = mpsc::channel();

    thread::spawn(move || {
        let mut sys = System::new_all();
        // Перша ініціалізація
        sys.refresh_all();

        loop {
            let uptime = System::uptime();
            let snapshot = SystemSnapshot::new(uptime);

            // Надсилаємо снепшот; якщо receiver закритий — завершуємо потік
            if tx.send(snapshot).is_err() {
                break;
            }

            thread::sleep(WORKER_INTERVAL);
        }
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_worker_sends_first_snapshot() {
        let rx = spawn();

        // Чекаємо перший снепшот (має прийти швидко)
        let snapshot = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Worker should send first snapshot within 2 seconds");

        // Перевіряємо, що uptime >= 0 (система працює)
        assert!(snapshot.uptime_secs > 0, "Uptime should be positive");
    }

    #[test]
    fn test_multiple_snapshots_increasing() {
        let rx = spawn();

        let first = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("First snapshot");
        let second = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Second snapshot");

        // Другий снепшот має бути пізніше (або рівний, якщо дуже швидко)
        assert!(
            second.uptime_secs >= first.uptime_secs,
            "Uptime should not decrease"
        );
    }
}

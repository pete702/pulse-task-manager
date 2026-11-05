//! Self-test GUI application for PULSE.

use std::sync::{mpsc, Arc};

use crate::core::spawn_telemetry_worker;
use crate::core::SystemSnapshot;
use crate::i18n::Key;
use crate::theme::apply_obsidian_theme;

/// Self-test application for `--self-test` GUI mode.
pub struct SelfTestApp {
    headless: bool,
    frames_rendered: u32,
    snapshot_received: bool,
    snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    latest_snapshot: Option<Arc<SystemSnapshot>>,
    test_passed: bool,
    /// Keep the worker alive to produce snapshots.
    _worker: crate::core::TelemetryWorker,
}

impl SelfTestApp {
    pub fn new(headless: bool) -> Self {
        let (worker, snapshot_rx) = spawn_telemetry_worker();
        Self {
            headless,
            frames_rendered: 0,
            snapshot_received: false,
            snapshot_rx,
            latest_snapshot: None,
            test_passed: false,
            _worker: worker,
        }
    }
}

impl eframe::App for SelfTestApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        apply_obsidian_theme(ctx);

        // Read snapshots
        let mut got_snapshot = false;
        while let Ok(snapshot) = self.snapshot_rx.try_recv() {
            self.latest_snapshot = Some(snapshot);
            got_snapshot = true;
        }

        if got_snapshot {
            self.snapshot_received = true;
        }

        if self.headless {
            // Headless CI mode: just check pulse_ram_mb > 0 after first snapshot
            if let Some(ref snapshot) = self.latest_snapshot {
                if snapshot.pulse_ram_mb > 0.0 {
                    self.test_passed = true;
                    println!("{}", Key::SelfTestPassedHeadless.t());
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            // Local GUI mode: wait for first snapshot, then render 5 frames, then check and close
            if self.snapshot_received {
                self.frames_rendered += 1;
                if self.frames_rendered >= 5 {
                    if let Some(ref snapshot) = self.latest_snapshot {
                        if snapshot.pulse_ram_mb > 0.0 {
                            self.test_passed = true;
                            println!("{}", Key::SelfTestPassedLive.t());
                        } else {
                            eprintln!("{}", Key::SelfTestFailed.t());
                        }
                    } else {
                        eprintln!("{}", Key::SelfTestFailed.t());
                    }
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                }
            } else {
                // Still waiting for first snapshot, request repaint to keep checking
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
    }
}

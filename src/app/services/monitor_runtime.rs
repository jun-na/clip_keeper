use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use rdev::{Event, EventType, Key};

use crate::app::services::clipboard_service::ClipboardService;
use crate::app::services::detectors::DoubleTapDetector;
use crate::app::services::ui_gateway::UiGateway;

const POLL_INTERVAL: Duration = Duration::from_millis(120);

// 監視ループ（クリップボード/ホットキー）を実行するランタイム。
pub struct MonitorRuntime {
    // クリップボード履歴を状態へ反映するサービス。
    clipboard_service: Arc<ClipboardService>,
    // UI表示更新を依頼するサービス。
    ui_gateway: Arc<UiGateway>,
}

impl MonitorRuntime {
    /// 監視に必要なサービスを受け取って生成する。
    pub fn new(clipboard_service: Arc<ClipboardService>, ui_gateway: Arc<UiGateway>) -> Self {
        Self {
            clipboard_service,
            ui_gateway,
        }
    }

    /// クリップボード監視スレッドとホットキー監視スレッドを開始する。
    pub fn start(&self) {
        self.start_clipboard_thread();
        self.start_hotkey_thread();
    }

    /// クリップボードをポーリングし、変化があれば状態とUIを更新する。
    fn start_clipboard_thread(&self) {
        // 別スレッドへ渡すため Arc を clone（実体コピーではなく共有参照を増やす）。
        // このスレッドでは ClipboardService を使って状態へ書き込む。
        let clipboard_service = self.clipboard_service.clone();
        // このスレッドでは UiGateway を使って履歴表示を更新する。
        let ui_gateway = self.ui_gateway.clone();

        thread::spawn(move || {
            let mut clipboard = Clipboard::new().ok();

            loop {
                if let Some(cb) = clipboard.as_mut() {
                    if let Ok(text) = cb.get_text() {
                        let changed = clipboard_service.push_clipboard_text(text);
                        if changed {
                            ui_gateway.refresh_history_model();
                        }
                    }
                } else {
                    clipboard = Clipboard::new().ok();
                }

                thread::sleep(POLL_INTERVAL);
            }
        });
    }

    /// グローバルホットキーを監視し、条件一致で履歴ウィンドウを表示する。
    fn start_hotkey_thread(&self) {
        // ホットキー検知時に履歴ウィンドウを開くため UiGateway を共有する。
        let ui_gateway = self.ui_gateway.clone();

        thread::spawn(move || {
            let mut shift_double_tap = DoubleTapDetector::default();
            let mut ctrl_double_tap = DoubleTapDetector::default();
            let mut ctrl_down = false;
            let mut shift_down = false;

            let callback = move |event: Event| match event.event_type {
                EventType::KeyPress(key) => match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_down = true;
                        if shift_double_tap.register_tap(Instant::now()) {
                            ui_gateway.show_history_window();
                        }
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        ctrl_down = true;
                        if ctrl_double_tap.register_tap(Instant::now()) {
                            ui_gateway.show_history_window();
                        }
                    }
                    Key::KeyH => {
                        if ctrl_down && shift_down {
                            ui_gateway.show_history_window();
                        }
                    }
                    _ => {}
                },
                EventType::KeyRelease(key) => match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_down = false;
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        ctrl_down = false;
                    }
                    _ => {}
                },
                _ => {}
            };

            if let Err(error) = rdev::listen(callback) {
                eprintln!("global hotkey listener failed: {error:?}");
            }
        });
    }
}

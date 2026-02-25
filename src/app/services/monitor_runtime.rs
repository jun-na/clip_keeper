use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use rdev::{Event, EventType, Key};

use crate::app::services::clipboard_service::ClipboardService;
use crate::app::services::detectors::DoubleTapDetector;
use crate::app::services::hotkey_logger::HotkeyLogger;
use crate::app::services::settings_service::SettingsService;
use crate::app::services::ui_gateway::UiGateway;

const POLL_INTERVAL: Duration = Duration::from_millis(120);

// 監視ループ（クリップボード/ホットキー）を実行するランタイム。
pub struct MonitorRuntime {
    // クリップボード履歴を状態へ反映するサービス。
    clipboard_service: Arc<ClipboardService>,
    // ホットキー設定を参照するサービス。
    settings_service: Arc<SettingsService>,
    // UI表示更新を依頼するサービス。
    ui_gateway: Arc<UiGateway>,
}

impl MonitorRuntime {
    /// 監視に必要なサービスを受け取って生成する。
    pub fn new(
        clipboard_service: Arc<ClipboardService>,
        settings_service: Arc<SettingsService>,
        ui_gateway: Arc<UiGateway>,
    ) -> Self {
        Self {
            clipboard_service,
            settings_service,
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
        let clipboard_service = self.clipboard_service.clone();
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
        // Arc クローンでスレッドに所有権を移す
        let settings_service = self.settings_service.clone();
        let ui_gateway = self.ui_gateway.clone();

        thread::spawn(move || {
            // HotkeyLogger はホットキースレッド専用・共有不要なのでここで生成する
            let logger = HotkeyLogger::new();
            // Shift キーのダブルタップ間隔を計測する検出器
            let mut shift_double_tap = DoubleTapDetector::default();
            // Ctrl キーのダブルタップ間隔を計測する検出器
            let mut ctrl_double_tap = DoubleTapDetector::default();

            // 修飾キーの現在の押下状態（キーリピートによる多重検知を防ぐ）
            let mut ctrl_down = false;
            let mut shift_down = false;

            // コンボキー（Ctrl+Shift+X など）の押下状態（同じく多重検知防止）
            let mut combo_key_down = false;

            // rdev に渡すイベントコールバック。キー押下／解放イベントを受け取る。
            let callback = move |event: Event| match event.event_type {
                // ─── キー押下イベント ───────────────────────────────────────
                EventType::KeyPress(key) => {
                    // 毎回最新の設定を取得することで、実行中の設定変更にも追従する
                    let settings = settings_service.current_hotkey_settings();

                    match key {
                        // ── Shift キー ──────────────────────────────────────
                        Key::ShiftLeft | Key::ShiftRight => {
                            // キーリピートによる二重登録を避けるため、
                            // すでに押下中の場合は何もしない
                            if !shift_down {
                                shift_down = true;
                                // モード 0（Shift 2回押し）のみダブルタップを判定する
                                if settings.hotkey_mode == 0
                                    && shift_double_tap.register_tap(Instant::now())
                                {
                                    logger.log(&format!("Shift double-tap ({key:?})"));
                                    ui_gateway.show_history_window();
                                }
                            }
                        }
                        // ── Ctrl キー ───────────────────────────────────────
                        Key::ControlLeft | Key::ControlRight => {
                            // Shift と同様にキーリピートを無視する
                            if !ctrl_down {
                                ctrl_down = true;
                                // モード 1（Ctrl 2回押し）のみダブルタップを判定する
                                if settings.hotkey_mode == 1
                                    && ctrl_double_tap.register_tap(Instant::now())
                                {
                                    logger.log(&format!("Ctrl double-tap ({key:?})"));
                                    ui_gateway.show_history_window();
                                }
                            }
                        }
                        // ── コンボキー（設定されたアルファベット／数字キー）───
                        _ => {
                            // モード 2（修飾キー+ホットキー）のみコンボ判定する
                            if settings.hotkey_mode == 2 && is_combo_key(key, &settings.combo_key) {
                                // キーリピートによる多重起動を防ぐ
                                if !combo_key_down {
                                    combo_key_down = true;
                                    // Ctrl 必須設定が OFF、または現在 Ctrl が押されていれば OK
                                    let ctrl_ok = !settings.combo_ctrl_required || ctrl_down;
                                    // Shift 必須設定が OFF、または現在 Shift が押されていれば OK
                                    let shift_ok = !settings.combo_shift_required || shift_down;
                                    // 両条件を満たす場合に履歴ウィンドウを表示する
                                    if ctrl_ok && shift_ok {
                                        logger.log(&format!(
                                            "Combo key ({key:?}) ctrl:{ctrl_down} shift:{shift_down}"
                                        ));
                                        ui_gateway.show_history_window();
                                    }
                                }
                            }
                        }
                    }
                }
                // ─── キー解放イベント ───────────────────────────────────────
                EventType::KeyRelease(key) => match key {
                    // Shift が離されたら押下フラグをリセットする
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_down = false;
                    }
                    // Ctrl が離されたら押下フラグをリセットする
                    Key::ControlLeft | Key::ControlRight => {
                        ctrl_down = false;
                    }
                    // コンボキーが離されたらそのフラグをリセットする
                    _ => {
                        let settings = settings_service.current_hotkey_settings();
                        if is_combo_key(key, &settings.combo_key) {
                            combo_key_down = false;
                        }
                    }
                },
                // キーボード以外のイベント（マウス等）は無視する
                _ => {}
            };

            // rdev のグローバルリスナーを開始する。
            // このブロッキング呼び出しはスレッド終了まで戻らない。
            if let Err(error) = rdev::listen(callback) {
                eprintln!("global hotkey listener failed: {error:?}");
            }
        });
    }
}

fn is_combo_key(key: Key, configured_key: &str) -> bool {
    if configured_key.is_empty() {
        return key == Key::KeyH;
    }

    let c = configured_key
        .chars()
        .next()
        .unwrap_or('H')
        .to_ascii_uppercase();
    match c {
        'A' => key == Key::KeyA,
        'B' => key == Key::KeyB,
        'C' => key == Key::KeyC,
        'D' => key == Key::KeyD,
        'E' => key == Key::KeyE,
        'F' => key == Key::KeyF,
        'G' => key == Key::KeyG,
        'H' => key == Key::KeyH,
        'I' => key == Key::KeyI,
        'J' => key == Key::KeyJ,
        'K' => key == Key::KeyK,
        'L' => key == Key::KeyL,
        'M' => key == Key::KeyM,
        'N' => key == Key::KeyN,
        'O' => key == Key::KeyO,
        'P' => key == Key::KeyP,
        'Q' => key == Key::KeyQ,
        'R' => key == Key::KeyR,
        'S' => key == Key::KeyS,
        'T' => key == Key::KeyT,
        'U' => key == Key::KeyU,
        'V' => key == Key::KeyV,
        'W' => key == Key::KeyW,
        'X' => key == Key::KeyX,
        'Y' => key == Key::KeyY,
        'Z' => key == Key::KeyZ,
        '0' => key == Key::Num0,
        '1' => key == Key::Num1,
        '2' => key == Key::Num2,
        '3' => key == Key::Num3,
        '4' => key == Key::Num4,
        '5' => key == Key::Num5,
        '6' => key == Key::Num6,
        '7' => key == Key::Num7,
        '8' => key == Key::Num8,
        '9' => key == Key::Num9,
        _ => key == Key::KeyH,
    }
}

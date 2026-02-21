use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Weak};

use crate::app::services::clipboard_service::ClipboardService;

// UIコンポーネント操作を集約するゲートウェイ。
pub struct UiGateway {
    // 履歴データ取得に利用するサービス。
    clipboard_service: Arc<ClipboardService>,
    history_window: Mutex<Option<Weak<crate::HistoryWindow>>>,
    settings_window: Mutex<Option<Weak<crate::SettingsWindow>>>,
}

impl UiGateway {
    /// UIゲートウェイを生成する。Window参照は後から attach する。
    pub fn new(clipboard_service: Arc<ClipboardService>) -> Self {
        Self {
            clipboard_service,
            history_window: Mutex::new(None),
            settings_window: Mutex::new(None),
        }
    }

    pub fn attach_windows(
        &self,
        history_window: &crate::HistoryWindow,
        settings_window: &crate::SettingsWindow,
    ) {
        // Window は Weak 参照で保持し、寿命は呼び出し側で管理する。
        {
            let mut history = self
                .history_window
                .lock()
                .expect("history window lock poisoned");
            *history = Some(history_window.as_weak());
        }
        {
            let mut settings = self
                .settings_window
                .lock()
                .expect("settings window lock poisoned");
            *settings = Some(settings_window.as_weak());
        }

        self.wire_callbacks();
    }

    /// Slint 側のコールバックと Rust 側の処理を接続する。
    pub fn wire_callbacks(&self) {
        let history_weak = self
            .history_window
            .lock()
            .expect("history window lock poisoned")
            .clone();
        let settings_weak = self
            .settings_window
            .lock()
            .expect("settings window lock poisoned")
            .clone();

        if let Some(history_weak) = history_weak {
            if let Some(history_window) = history_weak.upgrade() {
                history_window.on_request_hide({
                    let history_weak = history_weak.clone();
                    move || {
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                    }
                });

                if let Some(settings_weak) = settings_weak.clone() {
                    history_window.on_request_open_settings(move || {
                        if let Some(window) = settings_weak.upgrade() {
                            let _ = window.show();
                        }
                    });
                }
            }
        }

        if let Some(settings_weak) = settings_weak {
            if let Some(settings_window) = settings_weak.upgrade() {
                settings_window.on_request_hide({
                    let settings_weak = settings_weak.clone();
                    move || {
                        if let Some(window) = settings_weak.upgrade() {
                            let _ = window.hide();
                        }
                    }
                });
            }
        }
    }

    /// 履歴データをセットして履歴ウィンドウを表示する。
    pub fn show_history_window(&self) {
        // イベントループクロージャへ渡すためサービスを clone して共有する。
        let clipboard_service = self.clipboard_service.clone();
        let history_window = self
            .history_window
            .lock()
            .expect("history window lock poisoned")
            .clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(history_window) = history_window {
                if let Some(window) = history_window.upgrade() {
                    window.set_history_items(clipboard_service.history_model());
                    let _ = window.show();
                }
            }
        });
    }

    /// 設定ウィンドウを表示する。
    pub fn show_settings_window(&self) {
        let settings_window = self
            .settings_window
            .lock()
            .expect("settings window lock poisoned")
            .clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(settings_window) = settings_window {
                if let Some(window) = settings_window.upgrade() {
                    let _ = window.show();
                }
            }
        });
    }

    /// 履歴ウィンドウが開いている場合に表示データだけを更新する。
    pub fn refresh_history_model(&self) {
        // 履歴表示更新時も ClipboardService を通して状態を参照する。
        let clipboard_service = self.clipboard_service.clone();
        let history_window = self
            .history_window
            .lock()
            .expect("history window lock poisoned")
            .clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(history_window) = history_window {
                if let Some(window) = history_window.upgrade() {
                    window.set_history_items(clipboard_service.history_model());
                }
            }
        });
    }
}

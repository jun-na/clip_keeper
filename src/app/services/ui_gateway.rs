use std::sync::{Arc, Mutex};

use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{CloseRequestResponse, ComponentHandle, SharedString, Weak};

use crate::app::services::clipboard_service::ClipboardService;
use crate::app::services::settings_service::SettingsService;

// UIコンポーネント操作を集約するゲートウェイ。
pub struct UiGateway {
    // 履歴データ取得に利用するサービス。
    clipboard_service: Arc<ClipboardService>,
    // 設定表示と更新に利用するサービス。
    settings_service: Arc<SettingsService>,
    history_window: Mutex<Option<Weak<crate::HistoryWindow>>>,
    settings_window: Mutex<Option<Weak<crate::SettingsWindow>>>,
}

impl UiGateway {
    /// UIゲートウェイを生成する。Window参照は後から attach する。
    pub fn new(
        clipboard_service: Arc<ClipboardService>,
        settings_service: Arc<SettingsService>,
    ) -> Self {
        Self {
            clipboard_service,
            settings_service,
            history_window: Mutex::new(None),
            settings_window: Mutex::new(None),
        }
    }

    pub fn attach_windows(
        &self,
        history_window: &crate::HistoryWindow,
        settings_window: &crate::SettingsWindow,
    ) {
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
                history_window.on_request_select_history_item({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        if !clipboard_service.prepare_paste_from_history_index(index) {
                            return;
                        }
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                        clipboard_service.trigger_pending_paste();
                    }
                });

                history_window.window().on_close_requested({
                    let history_weak = history_weak.clone();
                    move || {
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                        CloseRequestResponse::KeepWindowShown
                    }
                });

                hook_hide_on_focus_lost(history_window.window());
            }
        }

        if let Some(settings_weak) = settings_weak {
            if let Some(settings_window) = settings_weak.upgrade() {
                settings_window.window().on_close_requested({
                    let settings_weak = settings_weak.clone();
                    move || {
                        if let Some(window) = settings_weak.upgrade() {
                            let _ = window.hide();
                        }
                        CloseRequestResponse::KeepWindowShown
                    }
                });

                settings_window.on_request_set_hotkey_ctrl_double_tap_enabled({
                    let settings_service = self.settings_service.clone();
                    move |enabled| {
                        settings_service.set_ctrl_double_tap_enabled(enabled);
                    }
                });

                settings_window.on_request_set_hotkey_shift_double_tap_enabled({
                    let settings_service = self.settings_service.clone();
                    move |enabled| {
                        settings_service.set_shift_double_tap_enabled(enabled);
                    }
                });

                settings_window.on_request_set_hotkey_combo_ctrl_required({
                    let settings_service = self.settings_service.clone();
                    move |enabled| {
                        settings_service.set_combo_ctrl_required(enabled);
                    }
                });

                settings_window.on_request_set_hotkey_combo_shift_required({
                    let settings_service = self.settings_service.clone();
                    move |enabled| {
                        settings_service.set_combo_shift_required(enabled);
                    }
                });

                settings_window.on_request_set_hotkey_combo_key({
                    let settings_service = self.settings_service.clone();
                    move |value| {
                        settings_service.set_combo_key(value.to_string());
                    }
                });

                hook_hide_on_focus_lost(settings_window.window());
            }
        }
    }

    /// 履歴データをセットして履歴ウィンドウを表示する。
    pub fn show_history_window(&self) {
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
                    // ウィンドウを開くたびに選択位置を先頭にリセット
                    window.set_selected_index(0);
                    let _ = window.show();
                    bring_to_front(window.window());
                }
            }
        });
    }

    /// 設定ウィンドウを表示する。
    pub fn show_settings_window(&self) {
        let settings_service = self.settings_service.clone();
        let settings_window = self
            .settings_window
            .lock()
            .expect("settings window lock poisoned")
            .clone();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(settings_window) = settings_window {
                if let Some(window) = settings_window.upgrade() {
                    let settings = settings_service.current_hotkey_settings();
                    window.set_hotkey_ctrl_double_tap_enabled(settings.ctrl_double_tap_enabled);
                    window.set_hotkey_shift_double_tap_enabled(settings.shift_double_tap_enabled);
                    window.set_hotkey_combo_ctrl_required(settings.combo_ctrl_required);
                    window.set_hotkey_combo_shift_required(settings.combo_shift_required);
                    window.set_hotkey_combo_key(SharedString::from(settings.combo_key));
                    let _ = window.show();
                    bring_to_front(window.window());
                }
            }
        });
    }

    /// 履歴ウィンドウが開いている場合に表示データだけを更新する。
    pub fn refresh_history_model(&self) {
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

fn hook_hide_on_focus_lost(window: &slint::Window) {
    window.on_winit_window_event(|slint_window, event| {
        if let winit::event::WindowEvent::Focused(false) = event {
            let _ = slint_window.hide();
        }
        EventResult::Propagate
    });
}

fn bring_to_front(window: &slint::Window) {
    window.with_winit_window(|winit_window: &winit::window::Window| {
        winit_window.focus_window();
        winit_window.request_user_attention(Some(winit::window::UserAttentionType::Informational));
    });
}

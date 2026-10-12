use std::sync::{Arc, Mutex};

use crate::app::domain::hotkey::HotkeyState;

use crate::app::domain::clipboard::ClipboardState;
use crate::app::domain::settings::SettingsState;

// ビルド時に生成する共有状態。
pub struct AppState {
    pub hotkey_log_enabled: Arc<Mutex<bool>>,
    pub history_window: Arc<Mutex<Option<slint::Weak<crate::HistoryWindow>>>>,
    pub settings_window: Arc<Mutex<Option<slint::Weak<crate::SettingsWindow>>>>,
    pub save_dialog_window: Arc<Mutex<Option<slint::Weak<crate::SaveDialogWindow>>>>,
    pub edit_saved_dialog_window: Arc<Mutex<Option<slint::Weak<crate::EditSavedDialogWindow>>>>,
    pub hotkey_state: Arc<Mutex<HotkeyState>>,
    // 複数スレッドから安全に更新できるよう Mutex で保護する。
    pub clipboard_state: Arc<Mutex<ClipboardState>>,
    // ホットキー設定を共有する状態。
    pub settings_state: Arc<Mutex<SettingsState>>,
}

impl AppState {
    /// 初期状態を保持した AppState を生成する。
    pub fn new() -> Self {
        Self {
            hotkey_log_enabled: Arc::new(Mutex::new(false)),
            history_window: Arc::new(Mutex::new(None)),
            settings_window: Arc::new(Mutex::new(None)),
            save_dialog_window: Arc::new(Mutex::new(None)),
            edit_saved_dialog_window: Arc::new(Mutex::new(None)),
            hotkey_state: Arc::new(Mutex::new(HotkeyState::default())),
            clipboard_state: Arc::new(Mutex::new(ClipboardState::new())),
            settings_state: Arc::new(Mutex::new(SettingsState::default())),
        }
    }
}

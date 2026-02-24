use std::sync::{Arc, Mutex};

use crate::app::states::app_state::AppState;
use crate::app::states::settings_state::SettingsState;

// アプリ状態の実体を保持するコンテキスト。
pub struct StateContext {
    // 複数スレッドから安全に更新できるよう Mutex で保護する。
    pub app_state: Arc<Mutex<AppState>>,
    // ホットキー設定を共有する状態。
    pub settings_state: Arc<Mutex<SettingsState>>,
}

impl StateContext {
    /// 初期状態を保持した StateContext を生成する。
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            app_state: Arc::new(Mutex::new(AppState::new())),
            settings_state: Arc::new(Mutex::new(SettingsState::default())),
        })
    }
}

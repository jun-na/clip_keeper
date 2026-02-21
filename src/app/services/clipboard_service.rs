use std::sync::Arc;

use slint::{ModelRc, SharedString};

use crate::app::contexts::state_context::StateContext;

// クリップボード履歴に関する状態読み書きを集約するサービス。
pub struct ClipboardService {
    state_context: Arc<StateContext>,
}

impl ClipboardService {
    /// StateContext を受け取り ClipboardService を生成する。
    pub fn new(state_context: Arc<StateContext>) -> Self {
        Self { state_context }
    }

    /// クリップボード文字列を履歴へ追加する。
    /// 追加が発生した場合 true を返す。
    pub fn push_clipboard_text(&self, text: String) -> bool {
        // 共有状態へ書き込み。
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.push_clipboard(text)
    }

    /// UI表示用の履歴モデルを取得する。
    pub fn history_model(&self) -> ModelRc<SharedString> {
        // UI表示用モデルとして履歴を読み出す。
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.history_model()
    }
}

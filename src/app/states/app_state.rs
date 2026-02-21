use std::collections::VecDeque;

use slint::{ModelRc, SharedString, VecModel};

const MAX_CLIPBOARD_ITEMS: usize = 50;

// クリップボード履歴のドメイン状態。
#[derive(Debug)]
pub struct AppState {
    history: VecDeque<String>,
    last_clipboard_text: Option<String>,
}

impl AppState {
    /// 空の履歴状態を生成する。
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            last_clipboard_text: None,
        }
    }

    /// 新しいクリップボード文字列を履歴へ反映する。
    /// 変化があった場合 true、重複等で変化なしなら false を返す。
    pub fn push_clipboard(&mut self, text: String) -> bool {
        if self.last_clipboard_text.as_deref() == Some(text.as_str()) {
            return false;
        }

        self.last_clipboard_text = Some(text.clone());
        self.history.retain(|item| item != &text);
        self.history.push_front(text);

        while self.history.len() > MAX_CLIPBOARD_ITEMS {
            self.history.pop_back();
        }

        true
    }

    /// Slint の List モデルへ変換してUIに渡せる形にする。
    pub fn history_model(&self) -> ModelRc<SharedString> {
        let rows: Vec<SharedString> = self
            .history
            .iter()
            .map(|item| {
                let normalized = item.replace(['\r', '\n'], " ");
                SharedString::from(normalized)
            })
            .collect();

        ModelRc::new(VecModel::from(rows))
    }
}

use std::sync::Arc;
use std::{env, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};
use slint::{ModelRc, SharedString};

use crate::app::contexts::state_context::StateContext;

// クリップボード履歴に関する状態読み書きを集約するサービス。
pub struct ClipboardService {
    state_context: Arc<StateContext>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedClipboardHistory {
    version: u32,
    items: Vec<String>,
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
        let changed = app_state.push_clipboard(text);
        if changed {
            // 履歴変更時にディスクへ保存する。
            if let Err(error) = self.save_history_to_disk_locked(&app_state) {
                eprintln!("failed to save clipboard history: {error}");
            }
        }

        changed
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

    /// アプリ起動時に履歴ファイルを読み込んで状態へ復元する。
    pub fn load_history_from_disk(&self) -> io::Result<()> {
        let path = history_file_path()?;
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let persisted: PersistedClipboardHistory =
            serde_json::from_str(&content).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid history json: {err}"),
                )
            })?;

        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.restore_history(persisted.items);
        Ok(())
    }

    fn save_history_to_disk_locked(
        &self,
        app_state: &crate::app::states::app_state::AppState,
    ) -> io::Result<()> {
        let path = history_file_path()?;
        let payload = PersistedClipboardHistory {
            version: 1,
            items: app_state.history_snapshot(),
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|err| io::Error::other(format!("serialize error: {err}")))?;
        fs::write(path, json)?;
        Ok(())
    }
}

fn history_file_path() -> io::Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve executable directory",
        )
    })?;
    Ok(exe_dir.join("clipboard_history.json"))
}

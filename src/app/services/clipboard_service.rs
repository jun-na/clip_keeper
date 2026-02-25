use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{env, fs, io, path::PathBuf};

use rdev::{simulate, EventType, Key};
use serde::{Deserialize, Serialize};
use slint::{ModelRc, SharedString};

use crate::app::contexts::state_context::StateContext;
use crate::app::states::app_state::{SavedGroup, SavedItem};
use crate::SavedEntry;

// クリップボード履歴に関する状態読み書きを集約するサービス。
pub struct ClipboardService {
    state_context: Arc<StateContext>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSavedItem {
    title: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSavedGroup {
    name: String,
    items: Vec<PersistedSavedItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedClipboardHistory {
    version: u32,
    items: Vec<String>,
    #[serde(default)]
    selected_index: i32,
    // v1 互換: フラットな saved_items があればデフォルトグループへ移行
    #[serde(default)]
    saved_items: Vec<PersistedSavedItem>,
    #[serde(default)]
    saved_groups: Vec<PersistedSavedGroup>,
    #[serde(default)]
    active_group: Option<String>,
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
        app_state.set_selected_index(persisted.selected_index);

        // グループ形式があればそちらを使い、なければ旧 saved_items をデフォルトグループへ移行
        if !persisted.saved_groups.is_empty() {
            app_state.restore_saved_groups(
                persisted
                    .saved_groups
                    .into_iter()
                    .map(|g| SavedGroup {
                        name: g.name,
                        items: g
                            .items
                            .into_iter()
                            .map(|i| SavedItem {
                                title: i.title,
                                content: i.content,
                            })
                            .collect(),
                    })
                    .collect(),
                persisted.active_group,
            );
        } else if !persisted.saved_items.is_empty() {
            // v1互換: フラットな saved_items をデフォルトグループへ
            app_state.restore_saved_groups(
                vec![SavedGroup {
                    name: "デフォルト".to_string(),
                    items: persisted
                        .saved_items
                        .into_iter()
                        .map(|i| SavedItem {
                            title: i.title,
                            content: i.content,
                        })
                        .collect(),
                }],
                None,
            );
        }
        Ok(())
    }

    /// 履歴で選択した項目を貼り付け待機状態にする。
    pub fn prepare_paste_from_history_index(&self, index: i32) -> bool {
        if index < 0 {
            return false;
        }

        let text = {
            let mut app_state = self
                .state_context
                .app_state
                .lock()
                .expect("app state lock poisoned");
            let Some(text) = app_state.history_item_at(index as usize) else {
                return false;
            };
            app_state.set_pending_paste(text.clone());
            // 選択インデックスを保存する。
            app_state.set_selected_index(index);
            if let Err(error) = self.save_history_to_disk_locked(&app_state) {
                eprintln!("failed to save selected index: {error}");
            }
            text
        };

        // 貼り付けキー送信前にクリップボード自体も更新しておく。
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }

        true
    }

    /// 貼り付け待機があれば、フォーカス遷移後に貼り付けキーを送信する。
    pub fn trigger_pending_paste(&self) {
        let pending = {
            let mut app_state = self
                .state_context
                .app_state
                .lock()
                .expect("app state lock poisoned");
            app_state.take_pending_paste()
        };

        if pending.is_none() {
            return;
        }

        thread::spawn(move || {
            // 履歴ウィンドウが隠れ、別アプリにフォーカスが移るまで待つ。
            thread::sleep(Duration::from_millis(180));
            if let Err(error) = simulate_paste_shortcut() {
                eprintln!("failed to simulate paste shortcut: {error:?}");
            }
        });
    }

    /// 保存された選択インデックスを取得する。
    pub fn selected_index(&self) -> i32 {
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.selected_index()
    }

    /// 履歴アイテムの内容を取得する。
    pub fn get_history_item_content(&self, index: i32) -> Option<String> {
        if index < 0 {
            return None;
        }
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.history_item_at(index as usize)
    }

    /// 先頭から指定インデックスまでの履歴を指定セパレータで連結し、貼り付け待機状態にする。
    pub fn prepare_bulk_paste(&self, up_to_index: i32, separator: &str) -> bool {
        if up_to_index < 0 {
            return false;
        }
        let joined = {
            let mut app_state = self
                .state_context
                .app_state
                .lock()
                .expect("app state lock poisoned");
            let items = app_state.history_items_up_to(up_to_index as usize);
            if items.is_empty() {
                return false;
            }
            let text = items.join(separator);
            app_state.set_pending_paste(text.clone());
            text
        };
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(joined);
        }
        true
    }

    /// グループ名を指定して保存アイテムを追加しディスクに永続化する。
    pub fn add_saved_item(&self, group: &str, title: String, content: String) {
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.add_saved_item(group, title, content);
        if let Err(error) = self.save_history_to_disk_locked(&app_state) {
            eprintln!("failed to save after adding saved item: {error}");
        }
    }

    /// アクティブグループの保存アイテムを削除しディスクに永続化する。
    pub fn remove_saved_item(&self, index: i32) {
        if index < 0 {
            return;
        }
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        if app_state.remove_saved_item(index as usize) {
            if let Err(error) = self.save_history_to_disk_locked(&app_state) {
                eprintln!("failed to save after removing saved item: {error}");
            }
        }
    }

    /// アクティブグループの保存アイテムの UI モデルを取得する。
    pub fn saved_items_model(&self) -> ModelRc<SavedEntry> {
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.saved_items_model()
    }

    /// グループ名の UI モデルを取得する。
    pub fn group_names_model(&self) -> ModelRc<SharedString> {
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.group_names_model()
    }

    /// グループ名の Vec を取得する。
    pub fn group_names(&self) -> Vec<String> {
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.group_names()
    }

    /// アクティブグループのインデックスを取得する。
    pub fn active_group_index(&self) -> i32 {
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.active_group_index()
    }

    /// アクティブグループを切り替える。
    pub fn set_active_group(&self, group: String) {
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.set_active_group(group);
        if let Err(error) = self.save_history_to_disk_locked(&app_state) {
            eprintln!("failed to save active group: {error}");
        }
    }

    /// 新しいグループを追加する。
    pub fn add_group(&self, name: String) {
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state.add_group(name);
        if let Err(error) = self.save_history_to_disk_locked(&app_state) {
            eprintln!("failed to save after adding group: {error}");
        }
    }

    /// 保存アイテムを選択して貼り付け待機状態にする。
    pub fn prepare_paste_from_saved_index(&self, index: i32) -> bool {
        if index < 0 {
            return false;
        }
        let text = {
            let mut app_state = self
                .state_context
                .app_state
                .lock()
                .expect("app state lock poisoned");
            let Some(text) = app_state.saved_item_content_at(index as usize) else {
                return false;
            };
            app_state.set_pending_paste(text.clone());
            text
        };
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
        true
    }

    /// 履歴アイテムを最新（先頭）へ移動する。
    pub fn move_history_to_front(&self, index: i32) {
        if index <= 0 {
            return;
        }
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        if app_state.move_to_front(index as usize) {
            if let Err(error) = self.save_history_to_disk_locked(&app_state) {
                eprintln!("failed to save after moving to front: {error}");
            }
        }
    }

    /// アクティブグループの保存アイテムのタイトルと内容を取得する。
    pub fn get_saved_item(&self, index: i32) -> Option<(String, String)> {
        if index < 0 {
            return None;
        }
        let app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        app_state
            .saved_item_at(index as usize)
            .map(|item| (item.title.clone(), item.content.clone()))
    }

    /// アクティブグループの保存アイテムを更新しディスクに永続化する。
    pub fn update_saved_item(&self, index: i32, title: String, content: String) {
        if index < 0 {
            return;
        }
        let mut app_state = self
            .state_context
            .app_state
            .lock()
            .expect("app state lock poisoned");
        if app_state.update_saved_item(index as usize, title, content) {
            if let Err(error) = self.save_history_to_disk_locked(&app_state) {
                eprintln!("failed to save after updating saved item: {error}");
            }
        }
    }

    fn save_history_to_disk_locked(
        &self,
        app_state: &crate::app::states::app_state::AppState,
    ) -> io::Result<()> {
        let path = history_file_path()?;
        let payload = PersistedClipboardHistory {
            version: 1,
            items: app_state.history_snapshot(),
            selected_index: app_state.selected_index(),
            saved_items: Vec::new(), // v2では空、saved_groupsを使用
            saved_groups: app_state
                .saved_groups_snapshot()
                .into_iter()
                .map(|g| PersistedSavedGroup {
                    name: g.name,
                    items: g
                        .items
                        .into_iter()
                        .map(|i| PersistedSavedItem {
                            title: i.title,
                            content: i.content,
                        })
                        .collect(),
                })
                .collect(),
            active_group: Some(app_state.active_group().to_string()),
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|err| io::Error::other(format!("serialize error: {err}")))?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn simulate_paste_shortcut() -> Result<(), rdev::SimulateError> {
    simulate(&EventType::KeyPress(Key::MetaLeft))?;
    simulate(&EventType::KeyPress(Key::KeyV))?;
    simulate(&EventType::KeyRelease(Key::KeyV))?;
    simulate(&EventType::KeyRelease(Key::MetaLeft))?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn simulate_paste_shortcut() -> Result<(), rdev::SimulateError> {
    simulate(&EventType::KeyPress(Key::ControlLeft))?;
    simulate(&EventType::KeyPress(Key::KeyV))?;
    simulate(&EventType::KeyRelease(Key::KeyV))?;
    simulate(&EventType::KeyRelease(Key::ControlLeft))?;
    Ok(())
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

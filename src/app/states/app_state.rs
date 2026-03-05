use std::collections::{BTreeMap, HashSet, VecDeque};

use slint::{ModelRc, SharedString, VecModel};

use crate::{HistoryEntry, SavedEntry};

const MAX_CLIPBOARD_ITEMS: usize = 1000;
const DEFAULT_GROUP_NAME: &str = "デフォルト";

/// 保存アイテムのドメインモデル。
#[derive(Debug, Clone)]
pub struct SavedItem {
    pub title: String,
    pub content: String,
}

/// グループ付き保存アイテム（永続化・復元用）。
#[derive(Debug, Clone)]
pub struct SavedGroup {
    pub name: String,
    pub items: Vec<SavedItem>,
}

// クリップボード履歴のドメイン状態。
#[derive(Debug)]
pub struct AppState {
    history: VecDeque<String>,
    used_items: HashSet<String>,
    last_clipboard_text: Option<String>,
    pending_paste_text: Option<String>,
    selected_index: i32,
    saved_groups: BTreeMap<String, Vec<SavedItem>>,
    group_order: Vec<String>,
    active_group: String,
}

impl AppState {
    /// 空の履歴状態を生成する。
    pub fn new() -> Self {
        let mut saved_groups = BTreeMap::new();
        saved_groups.insert(DEFAULT_GROUP_NAME.to_string(), Vec::new());
        Self {
            history: VecDeque::new(),
            used_items: HashSet::new(),
            last_clipboard_text: None,
            pending_paste_text: None,
            selected_index: 0,
            saved_groups,
            group_order: vec![DEFAULT_GROUP_NAME.to_string()],
            active_group: DEFAULT_GROUP_NAME.to_string(),
        }
    }

    /// 新しいクリップボード文字列を履歴へ反映する。
    /// 変化があった場合 true、重複等で変化なしなら false を返す。
    pub fn push_clipboard(&mut self, text: String) -> bool {
        if self.last_clipboard_text.as_deref() == Some(text.as_str()) {
            return false;
        }

        self.last_clipboard_text = Some(text.clone());

        // 既に履歴に存在する場合は位置を変えずスキップする。
        if self.history.contains(&text) {
            return false;
        }

        self.history.push_front(text);

        while self.history.len() > MAX_CLIPBOARD_ITEMS {
            if let Some(removed) = self.history.pop_back() {
                self.used_items.remove(&removed);
            }
        }

        true
    }

    /// Slint の List モデルへ変換してUIに渡せる形にする。
    pub fn history_model(&self) -> ModelRc<HistoryEntry> {
        let rows: Vec<HistoryEntry> = self
            .history
            .iter()
            .map(|item| {
                let normalized = item.replace(['\r', '\n'], " ");
                let display = if normalized.chars().count() > 120 {
                    let truncated: String = normalized.chars().take(120).collect();
                    format!("{truncated}…")
                } else {
                    normalized
                };
                HistoryEntry {
                    text: SharedString::from(display),
                    used: self.used_items.contains(item),
                }
            })
            .collect();

        ModelRc::new(VecModel::from(rows))
    }

    /// 現在の履歴を永続化向けにコピーする（先頭が最新）。
    pub fn history_snapshot(&self) -> Vec<String> {
        self.history.iter().cloned().collect()
    }

    /// 永続化データから履歴を復元する。
    pub fn restore_history(&mut self, items: Vec<String>) {
        let mut seen = HashSet::new();
        let mut restored = VecDeque::new();

        for item in items {
            if item.is_empty() {
                continue;
            }
            if seen.insert(item.clone()) {
                restored.push_back(item);
            }
            if restored.len() >= MAX_CLIPBOARD_ITEMS {
                break;
            }
        }

        self.last_clipboard_text = restored.front().cloned();
        self.history = restored;
    }

    /// アイテムを使用済みとしてマークする。
    pub fn mark_as_used(&mut self, text: &str) {
        self.used_items.insert(text.to_string());
    }

    /// 使用済みアイテムのスナップショットを返す（永続化用）。
    pub fn used_items_snapshot(&self) -> Vec<String> {
        self.used_items.iter().cloned().collect()
    }

    /// 永続化データから使用済みアイテムを復元する。
    pub fn restore_used_items(&mut self, items: Vec<String>) {
        self.used_items = items.into_iter().collect();
    }

    /// 指定インデックスの履歴文字列を取得する。
    pub fn history_item_at(&self, index: usize) -> Option<String> {
        self.history.get(index).cloned()
    }

    /// 先頭から指定インデックスまでの履歴アイテムを取得する（0..=index）。
    pub fn history_items_up_to(&self, index: usize) -> Vec<String> {
        self.history.iter().take(index + 1).cloned().collect()
    }

    /// 指定インデックスの履歴アイテムを最新（先頭）へ移動する。
    /// 移動が発生した場合 true を返す。
    pub fn move_to_front(&mut self, index: usize) -> bool {
        if index == 0 || index >= self.history.len() {
            return false;
        }
        if let Some(item) = self.history.remove(index) {
            self.history.push_front(item);
            self.last_clipboard_text = self.history.front().cloned();
            true
        } else {
            false
        }
    }

    /// 次にアクティブになるウィンドウへ貼り付ける文字列をセットする。
    pub fn set_pending_paste(&mut self, text: String) {
        self.pending_paste_text = Some(text);
    }

    /// 貼り付け待機中の文字列を取り出す（取り出し後はクリア）。
    pub fn take_pending_paste(&mut self) -> Option<String> {
        self.pending_paste_text.take()
    }

    /// 選択中のインデックスを取得する。
    pub fn selected_index(&self) -> i32 {
        self.selected_index
    }

    /// 選択中のインデックスを更新する。
    pub fn set_selected_index(&mut self, index: i32) {
        self.selected_index = index;
    }

    // ── 保存アイテム（グループ管理） ──

    /// 現在アクティブなグループ名を返す。
    pub fn active_group(&self) -> &str {
        &self.active_group
    }

    /// アクティブなグループを切り替える。
    pub fn set_active_group(&mut self, group: String) {
        // 存在しなければ作成
        if !self.saved_groups.contains_key(&group) {
            self.saved_groups.insert(group.clone(), Vec::new());
            self.group_order.push(group.clone());
        }
        self.active_group = group;
    }

    /// グループ名の一覧を返す（追加順）。
    pub fn group_names(&self) -> Vec<String> {
        self.group_order.clone()
    }

    /// 新しいグループを追加する。既に存在する場合は何もしない。
    pub fn add_group(&mut self, name: String) -> bool {
        let normalized = name.trim();
        if normalized.is_empty() || self.saved_groups.contains_key(normalized) {
            return false;
        }
        self.saved_groups.insert(normalized.to_string(), Vec::new());
        self.group_order.push(normalized.to_string());
        true
    }

    /// 指定インデックスのグループ名を変更する。
    pub fn rename_group(&mut self, index: usize, new_name: String) -> bool {
        let normalized = new_name.trim();
        if normalized.is_empty() {
            return false;
        }

        let Some(old_name) = self.group_order.get(index).cloned() else {
            return false;
        };
        if old_name == DEFAULT_GROUP_NAME {
            return false;
        }
        if old_name != normalized && self.saved_groups.contains_key(normalized) {
            return false;
        }
        if old_name == normalized {
            return true;
        }

        let Some(items) = self.saved_groups.remove(&old_name) else {
            return false;
        };
        self.saved_groups.insert(normalized.to_string(), items);
        if let Some(slot) = self.group_order.get_mut(index) {
            *slot = normalized.to_string();
        }
        if self.active_group == old_name {
            self.active_group = normalized.to_string();
        }
        true
    }

    /// 指定インデックスのグループを削除し、アイテムはデフォルトへ移動する。
    pub fn delete_group(&mut self, index: usize) -> bool {
        let Some(target_name) = self.group_order.get(index).cloned() else {
            return false;
        };
        if target_name == DEFAULT_GROUP_NAME {
            return false;
        }

        let Some(removed_items) = self.saved_groups.remove(&target_name) else {
            return false;
        };
        self.group_order.retain(|name| name != &target_name);
        self.saved_groups
            .entry(DEFAULT_GROUP_NAME.to_string())
            .or_default()
            .extend(removed_items);

        if self.active_group == target_name {
            self.active_group = DEFAULT_GROUP_NAME.to_string();
        }
        true
    }

    /// アクティブグループにアイテムを追加する。
    pub fn add_saved_item(&mut self, group: &str, title: String, content: String) {
        if !self.saved_groups.contains_key(group) {
            self.saved_groups.insert(group.to_string(), Vec::new());
            self.group_order.push(group.to_string());
        }
        let items = self
            .saved_groups
            .get_mut(group)
            .expect("group should exist after initialization");
        items.push(SavedItem { title, content });
    }

    /// アクティブグループの指定インデックスを削除する。
    pub fn remove_saved_item(&mut self, index: usize) -> bool {
        if let Some(items) = self.saved_groups.get_mut(&self.active_group) {
            if index < items.len() {
                items.remove(index);
                return true;
            }
        }
        false
    }

    /// 全グループのスナップショットを返す（永続化用）。
    pub fn saved_groups_snapshot(&self) -> Vec<SavedGroup> {
        self.group_order
            .iter()
            .filter_map(|name| {
                self.saved_groups.get(name).map(|items| SavedGroup {
                    name: name.clone(),
                    items: items.clone(),
                })
            })
            .collect()
    }

    /// 永続化データからグループを復元する。
    pub fn restore_saved_groups(&mut self, groups: Vec<SavedGroup>, active_group: Option<String>) {
        self.saved_groups.clear();
        self.group_order.clear();
        for group in groups {
            if self.saved_groups.contains_key(&group.name) {
                continue;
            }
            self.group_order.push(group.name.clone());
            self.saved_groups.insert(group.name, group.items);
        }
        // デフォルトグループが必ず存在するようにする
        if !self.saved_groups.contains_key(DEFAULT_GROUP_NAME) {
            self.saved_groups
                .insert(DEFAULT_GROUP_NAME.to_string(), Vec::new());
        }
        if !self.group_order.iter().any(|name| name == DEFAULT_GROUP_NAME) {
            self.group_order.push(DEFAULT_GROUP_NAME.to_string());
        }
        // active_group の復元
        if let Some(ag) = active_group {
            if self.saved_groups.contains_key(&ag) {
                self.active_group = ag;
            } else {
                self.active_group = DEFAULT_GROUP_NAME.to_string();
            }
        }
    }

    /// アクティブグループの保存アイテムを Slint モデルへ変換する。
    pub fn saved_items_model(&self) -> ModelRc<SavedEntry> {
        let items = self
            .saved_groups
            .get(&self.active_group)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let rows: Vec<SavedEntry> = items
            .iter()
            .map(|item| SavedEntry {
                title: SharedString::from(item.title.as_str()),
                content: SharedString::from(item.content.as_str()),
            })
            .collect();
        ModelRc::new(VecModel::from(rows))
    }

    /// グループ名リストを Slint モデルへ変換する。
    pub fn group_names_model(&self) -> ModelRc<SharedString> {
        let names: Vec<SharedString> = self
            .group_order
            .iter()
            .map(|k| SharedString::from(k.as_str()))
            .collect();
        ModelRc::new(VecModel::from(names))
    }

    /// アクティブグループのグループ名リスト中のインデックスを返す。
    pub fn active_group_index(&self) -> i32 {
        self.group_order
            .iter()
            .position(|k| k == &self.active_group)
            .map(|i| i as i32)
            .unwrap_or(0)
    }

    /// アクティブグループの指定インデックスの保存アイテムの内容を取得する。
    pub fn saved_item_content_at(&self, index: usize) -> Option<String> {
        self.saved_groups
            .get(&self.active_group)
            .and_then(|items| items.get(index))
            .map(|item| item.content.clone())
    }

    /// アクティブグループの指定インデックスの保存アイテムを取得する。
    pub fn saved_item_at(&self, index: usize) -> Option<&SavedItem> {
        self.saved_groups
            .get(&self.active_group)
            .and_then(|items| items.get(index))
    }

    /// アクティブグループの指定インデックスの保存アイテムを更新する。
    pub fn update_saved_item(&mut self, index: usize, title: String, content: String) -> bool {
        if let Some(items) = self.saved_groups.get_mut(&self.active_group) {
            if let Some(item) = items.get_mut(index) {
                item.title = title;
                item.content = content;
                return true;
            }
        }
        false
    }
}

use std::sync::{Arc, Mutex};

use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{CloseRequestResponse, ComponentHandle, Model, SharedString, Weak};

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
    save_dialog_window: Mutex<Option<Weak<crate::SaveDialogWindow>>>,
    edit_saved_dialog_window: Mutex<Option<Weak<crate::EditSavedDialogWindow>>>,
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
            save_dialog_window: Mutex::new(None),
            edit_saved_dialog_window: Mutex::new(None),
        }
    }

    pub fn attach_windows(
        &self,
        history_window: &crate::HistoryWindow,
        settings_window: &crate::SettingsWindow,
        save_dialog_window: &crate::SaveDialogWindow,
        edit_saved_dialog_window: &crate::EditSavedDialogWindow,
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
        {
            let mut save_dialog = self
                .save_dialog_window
                .lock()
                .expect("save dialog window lock poisoned");
            *save_dialog = Some(save_dialog_window.as_weak());
        }
        {
            let mut edit_saved_dialog = self
                .edit_saved_dialog_window
                .lock()
                .expect("edit saved dialog lock poisoned");
            *edit_saved_dialog = Some(edit_saved_dialog_window.as_weak());
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
        let save_dialog_weak = self
            .save_dialog_window
            .lock()
            .expect("save dialog window lock poisoned")
            .clone();
        let edit_saved_dialog_weak = self
            .edit_saved_dialog_window
            .lock()
            .expect("edit saved dialog lock poisoned")
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

                // 右クリック「保存」→ 保存ダイアログを開く
                history_window.on_request_save_history_item({
                    let clipboard_service = self.clipboard_service.clone();
                    let save_dialog_weak = save_dialog_weak.clone();
                    move |index| {
                        let Some(content) = clipboard_service.get_history_item_content(index) else {
                            return;
                        };
                        let title = generate_title_from_content(&content);
                        if let Some(dialog) = save_dialog_weak.as_ref().and_then(|w| w.upgrade()) {
                            dialog.set_save_title(SharedString::from(&title));
                            dialog.set_save_content(SharedString::from(&content));
                            dialog.set_group_names(clipboard_service.group_names_model());
                            dialog.set_selected_group_index(clipboard_service.active_group_index());
                            dialog.set_creating_new_group(false);
                            dialog.set_new_group_name(SharedString::default());
                            let _ = dialog.show();
                            bring_to_front(dialog.window());
                        }
                    }
                });

                // 右クリック「最新に移動」
                history_window.on_request_move_to_front({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        clipboard_service.move_history_to_front(index);
                        if let Some(window) = history_weak.upgrade() {
                            window.set_history_items(clipboard_service.history_model());
                            window.set_selected_index(0);
                        }
                    }
                });

                // 一括貼り付け（連結）
                history_window.on_request_bulk_paste_concat({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        if !clipboard_service.prepare_bulk_paste(index, "") {
                            return;
                        }
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                        clipboard_service.trigger_pending_paste();
                    }
                });

                // 一括貼り付け（Tab挿入）
                history_window.on_request_bulk_paste_tab({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        if !clipboard_service.prepare_bulk_paste(index, "\t") {
                            return;
                        }
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                        clipboard_service.trigger_pending_paste();
                    }
                });

                // 一括貼り付け（改行挿入）
                history_window.on_request_bulk_paste_newline({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        if !clipboard_service.prepare_bulk_paste(index, "\n") {
                            return;
                        }
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                        clipboard_service.trigger_pending_paste();
                    }
                });

                // 保存タブ右クリック「編集」
                history_window.on_request_edit_saved_item({
                    let clipboard_service = self.clipboard_service.clone();
                    let edit_saved_dialog_weak = edit_saved_dialog_weak.clone();
                    move |index| {
                        let Some((title, content)) = clipboard_service.get_saved_item(index) else {
                            return;
                        };
                        if let Some(dialog) = edit_saved_dialog_weak.as_ref().and_then(|w| w.upgrade()) {
                            dialog.set_edit_index(index);
                            dialog.set_edit_title(SharedString::from(&title));
                            dialog.set_edit_content(SharedString::from(&content));
                            let _ = dialog.show();
                            bring_to_front(dialog.window());
                        }
                    }
                });

                // グループ切り替え
                history_window.on_request_switch_group({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |group_index| {
                        let group_names = clipboard_service.group_names();
                        if let Some(name) = group_names.get(group_index as usize) {
                            clipboard_service.set_active_group(name.clone());
                            if let Some(window) = history_weak.upgrade() {
                                window.set_saved_items(clipboard_service.saved_items_model());
                                window.set_saved_selected_index(0);
                            }
                        }
                    }
                });

                // 保存済みアイテムをクリック → 貼り付け
                history_window.on_request_select_saved_item({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        if !clipboard_service.prepare_paste_from_saved_index(index) {
                            return;
                        }
                        if let Some(window) = history_weak.upgrade() {
                            let _ = window.hide();
                        }
                        clipboard_service.trigger_pending_paste();
                    }
                });

                // 保存済みアイテムを削除
                history_window.on_request_delete_saved_item({
                    let clipboard_service = self.clipboard_service.clone();
                    let history_weak = history_weak.clone();
                    move |index| {
                        clipboard_service.remove_saved_item(index);
                        if let Some(window) = history_weak.upgrade() {
                            window.set_saved_items(clipboard_service.saved_items_model());
                        }
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

        // 保存ダイアログのコールバック
        if let Some(save_dialog_weak) = save_dialog_weak {
            if let Some(save_dialog) = save_dialog_weak.upgrade() {
                save_dialog.on_request_confirm_save({
                    let clipboard_service = self.clipboard_service.clone();
                    let save_dialog_weak = save_dialog_weak.clone();
                    let history_weak = self
                        .history_window
                        .lock()
                        .expect("history window lock poisoned")
                        .clone();
                    move |group, title, content| {
                        let group_name = group.to_string();
                        clipboard_service.add_group(group_name.clone());
                        clipboard_service.add_saved_item(
                            &group_name,
                            title.to_string(),
                            content.to_string(),
                        );
                        if let Some(dialog) = save_dialog_weak.upgrade() {
                            let _ = dialog.hide();
                        }
                        // 履歴ウィンドウの保存リスト・グループも更新
                        if let Some(history_weak) = &history_weak {
                            if let Some(window) = history_weak.upgrade() {
                                window.set_group_names(clipboard_service.group_names_model());
                                window.set_saved_items(clipboard_service.saved_items_model());
                            }
                        }
                    }
                });

                save_dialog.on_request_cancel_save({
                    let save_dialog_weak = save_dialog_weak.clone();
                    move || {
                        if let Some(dialog) = save_dialog_weak.upgrade() {
                            let _ = dialog.hide();
                        }
                    }
                });

                save_dialog.window().on_close_requested({
                    let save_dialog_weak = save_dialog_weak.clone();
                    move || {
                        if let Some(dialog) = save_dialog_weak.upgrade() {
                            let _ = dialog.hide();
                        }
                        CloseRequestResponse::KeepWindowShown
                    }
                });
            }
        }

        // 編集ダイアログのコールバック
        if let Some(edit_saved_dialog_weak) = edit_saved_dialog_weak {
            if let Some(edit_dialog) = edit_saved_dialog_weak.upgrade() {
                edit_dialog.on_request_confirm_edit({
                    let clipboard_service = self.clipboard_service.clone();
                    let edit_saved_dialog_weak = edit_saved_dialog_weak.clone();
                    let history_weak = self
                        .history_window
                        .lock()
                        .expect("history window lock poisoned")
                        .clone();
                    move |index, title, content| {
                        clipboard_service.update_saved_item(
                            index,
                            title.to_string(),
                            content.to_string(),
                        );
                        if let Some(dialog) = edit_saved_dialog_weak.upgrade() {
                            let _ = dialog.hide();
                        }
                        if let Some(history_weak) = &history_weak {
                            if let Some(window) = history_weak.upgrade() {
                                window.set_saved_items(clipboard_service.saved_items_model());
                            }
                        }
                    }
                });

                edit_dialog.on_request_cancel_edit({
                    let edit_saved_dialog_weak = edit_saved_dialog_weak.clone();
                    move || {
                        if let Some(dialog) = edit_saved_dialog_weak.upgrade() {
                            let _ = dialog.hide();
                        }
                    }
                });

                edit_dialog.window().on_close_requested({
                    let edit_saved_dialog_weak = edit_saved_dialog_weak.clone();
                    move || {
                        if let Some(dialog) = edit_saved_dialog_weak.upgrade() {
                            let _ = dialog.hide();
                        }
                        CloseRequestResponse::KeepWindowShown
                    }
                });
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
                    window.set_saved_items(clipboard_service.saved_items_model());
                    window.set_group_names(clipboard_service.group_names_model());
                    window.set_active_group_index(clipboard_service.active_group_index());
                    // 前回保存された選択位置を復元する
                    let saved_index = clipboard_service.selected_index();
                    let item_count = window.get_history_items().row_count() as i32;
                    let index = if item_count > 0 {
                        saved_index.clamp(0, item_count - 1)
                    } else {
                        0
                    };
                    window.set_selected_index(index);
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

/// コンテンツの先頭行からタイトルを自動生成する。
fn generate_title_from_content(content: &str) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let truncated: String = first_line.chars().take(30).collect();
    if truncated.chars().count() < first_line.chars().count() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

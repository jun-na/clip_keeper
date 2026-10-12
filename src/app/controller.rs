use std::sync::Arc;

use super::request::Request;
use crate::app::services::{
    clipboard::ClipboardService, logging::LoggingService, settings::SettingsService, ui::UiService,
};
use crate::app::Application;

/// UIスレッド上でRequestを振り分け、サービスの実行順序を制御する。
pub struct Controller<D = Application> {
    services: Arc<D>,
}

impl<D: ClipboardService + SettingsService + UiService + LoggingService> Controller<D> {
    pub fn new(services: Arc<D>) -> Self {
        Self { services }
    }

    pub(super) fn dispatch(&self, request: Request) {
        let deps = self.services.as_ref();
        match request {
            Request::HideHistory => deps.hide_history_window(),
            Request::PasteHistory { index } => {
                if deps.prepare_paste_from_history_index(index) {
                    deps.hide_history_window();
                    deps.trigger_pending_paste();
                }
            }
            Request::OpenSaveDialog { index } => {
                if let Some(content) = deps.get_history_item_content(index) {
                    deps.show_save_dialog(&content);
                }
            }
            Request::MoveHistoryToFront { index } => {
                deps.move_history_to_front(index);
                deps.refresh_history(Some(0));
            }
            Request::BulkPasteConcat { index } => self.bulk_paste(index, ""),
            Request::BulkPasteTab { index } => self.bulk_paste(index, "\t"),
            Request::BulkPasteNewline { index } => self.bulk_paste(index, "\n"),
            Request::OpenEditDialog { index } => {
                if let Some((title, content)) = deps.get_saved_item(index) {
                    deps.show_edit_dialog(index, &title, &content);
                }
            }
            Request::SwitchGroup { group_index } => {
                if let Some(name) = deps.group_names().get(group_index as usize) {
                    deps.set_active_group(name.clone());
                    deps.refresh_saved_groups(true);
                }
            }
            Request::PasteSaved { index } => {
                if deps.prepare_paste_from_saved_index(index) {
                    deps.hide_history_window();
                    deps.trigger_pending_paste();
                }
            }
            Request::DeleteSaved { index } => {
                deps.remove_saved_item(index);
                deps.refresh_saved_items();
            }
            Request::SetHotkeyMode { mode } => deps.set_hotkey_mode(mode),
            Request::SetComboCtrl { enabled } => deps.set_combo_ctrl_required(enabled),
            Request::SetComboShift { enabled } => deps.set_combo_shift_required(enabled),
            Request::SetComboKey { value } => deps.set_combo_key(value),
            Request::SaveItem {
                group,
                title,
                content,
            } => {
                deps.add_saved_item(&group, title, content);
                deps.hide_save_dialog();
                deps.refresh_saved_groups(false);
            }
            Request::HideSaveDialog => deps.hide_save_dialog(),
            Request::EditSavedItem {
                index,
                title,
                content,
            } => {
                deps.update_saved_item(index, title, content);
                deps.hide_edit_dialog();
                deps.refresh_saved_items();
            }
            Request::HideEditDialog => deps.hide_edit_dialog(),
            Request::ClipboardObserved { text } => {
                if deps.push_clipboard_text(text) {
                    deps.refresh_history(None);
                }
            }
            Request::HotkeyEvent { event, at } => {
                if let Some(trigger) = deps.handle_hotkey_event(event, at) {
                    deps.log_hotkey(&trigger);
                    deps.show_history_window();
                }
            }
            Request::ShowHistory => deps.show_history_window(),
            Request::ShowSettings => deps.show_settings_window(),
            Request::HideSettings => deps.hide_settings_window(),
            Request::Quit => deps.quit_ui(),
        }
    }

    fn bulk_paste(&self, index: i32, separator: &str) {
        if self.services.prepare_bulk_paste(index, separator) {
            self.services.hide_history_window();
            self.services.trigger_pending_paste();
        }
    }
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;

use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{CloseRequestResponse, ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::app::request::{Request, RequestSender};

use crate::app::services::{clipboard::ClipboardService, settings::SettingsService};
use crate::app::AccessState;

/// 画面の強参照を保持するセッション。破棄すると画面も解放する。
pub struct UiSession {
    _history: crate::HistoryWindow,
    _settings: crate::SettingsWindow,
    _save: crate::SaveDialogWindow,
    _edit: crate::EditSavedDialogWindow,
}

// 画面操作はUIスレッド上で実行。RequestはMediatorで転送する。
#[entrait::entrait(pub UiService)]
mod operations {
    use super::*;

    pub fn initialize_ui(
        deps: &impl AccessState,
        send_request: RequestSender,
    ) -> Result<UiSession, slint::PlatformError> {
        let history = crate::HistoryWindow::new()?;
        history.hide()?;
        let settings = crate::SettingsWindow::new()?;
        settings.hide()?;
        let save = crate::SaveDialogWindow::new()?;
        save.hide()?;
        let edit = crate::EditSavedDialogWindow::new()?;
        edit.hide()?;

        attach_windows(deps, &history, &settings, &save, &edit);
        connect_callbacks(send_request, &history, &settings, &save, &edit);
        Ok(UiSession {
            _history: history,
            _settings: settings,
            _save: save,
            _edit: edit,
        })
    }

    pub fn hide_history_window(deps: &impl AccessState) {
        if let Some(window) = history_window(deps) {
            let _ = window.hide();
        }
    }

    pub fn hide_settings_window(deps: &impl AccessState) {
        if let Some(window) = settings_window(deps) {
            let _ = window.hide();
        }
    }

    pub fn hide_save_dialog(deps: &impl AccessState) {
        if let Some(dialog) = save_dialog(deps) {
            let _ = dialog.hide();
        }
    }

    pub fn hide_edit_dialog(deps: &impl AccessState) {
        if let Some(dialog) = edit_dialog(deps) {
            let _ = dialog.hide();
        }
    }

    pub fn show_history_window(deps: &(impl AccessState + ClipboardService)) {
        if let Some(window) = history_window(deps) {
            window.set_history_items(history_model(deps));
            window.set_saved_items(saved_items_model(deps));
            window.set_group_names(group_names_model(deps));
            window.set_active_group_index(deps.active_group_index());
            let saved_index = deps.selected_index();
            let item_count = window.get_history_items().row_count() as i32;
            let index = if item_count > 0 {
                saved_index.clamp(0, item_count - 1)
            } else {
                0
            };
            window.set_selected_index(index);
            let _ = window.show();
            set_app_icon(window.window());
            bring_to_front(window.window());
        }
    }

    pub fn show_settings_window(deps: &(impl AccessState + SettingsService)) {
        if let Some(window) = settings_window(deps) {
            let settings = deps.current_hotkey_settings();
            window.set_hotkey_mode(settings.hotkey_mode);
            window.set_hotkey_combo_ctrl_required(settings.combo_ctrl_required);
            window.set_hotkey_combo_shift_required(settings.combo_shift_required);
            window.set_hotkey_combo_key(SharedString::from(settings.combo_key));
            let _ = window.show();
            set_app_icon(window.window());
            bring_to_front(window.window());
        }
    }

    pub fn show_save_dialog(deps: &(impl AccessState + ClipboardService), content: &str) {
        if let Some(dialog) = save_dialog(deps) {
            let title = generate_title_from_content(content);
            dialog.set_save_title(SharedString::from(&title));
            dialog.set_save_content(SharedString::from(content));
            dialog.set_group_names(group_names_model(deps));
            dialog.set_selected_group_index(deps.active_group_index());
            let _ = dialog.show();
            set_app_icon(dialog.window());
            bring_to_front(dialog.window());
        }
    }

    pub fn show_edit_dialog(deps: &impl AccessState, index: i32, title: &str, content: &str) {
        if let Some(dialog) = edit_dialog(deps) {
            dialog.set_edit_index(index);
            dialog.set_edit_title(SharedString::from(title));
            dialog.set_edit_content(SharedString::from(content));
            let _ = dialog.show();
            set_app_icon(dialog.window());
            bring_to_front(dialog.window());
        }
    }

    pub fn refresh_history(
        deps: &(impl AccessState + ClipboardService),
        selected_index: Option<i32>,
    ) {
        if let Some(window) = history_window(deps) {
            window.set_history_items(history_model(deps));
            if let Some(index) = selected_index {
                window.set_selected_index(index);
            }
        }
    }

    pub fn refresh_saved_items(deps: &(impl AccessState + ClipboardService)) {
        if let Some(window) = history_window(deps) {
            window.set_saved_items(saved_items_model(deps));
        }
    }

    pub fn refresh_saved_groups(
        deps: &(impl AccessState + ClipboardService),
        reset_selection: bool,
    ) {
        if let Some(window) = history_window(deps) {
            window.set_group_names(group_names_model(deps));
            window.set_active_group_index(deps.active_group_index());
            window.set_saved_items(saved_items_model(deps));
            if reset_selection {
                window.set_saved_selected_index(0);
            }
        }
    }

    pub fn quit_ui(_deps: &impl AccessState) {
        if let Err(error) = slint::quit_event_loop() {
            eprintln!("failed to quit event loop: {error}");
        }
    }
}

fn history_window(deps: &impl AccessState) -> Option<crate::HistoryWindow> {
    let weak = deps
        .state()
        .history_window
        .lock()
        .expect("window lock poisoned")
        .clone();
    weak.and_then(|window| window.upgrade())
}

fn settings_window(deps: &impl AccessState) -> Option<crate::SettingsWindow> {
    let weak = deps
        .state()
        .settings_window
        .lock()
        .expect("window lock poisoned")
        .clone();
    weak.and_then(|window| window.upgrade())
}

fn save_dialog(deps: &impl AccessState) -> Option<crate::SaveDialogWindow> {
    let weak = deps
        .state()
        .save_dialog_window
        .lock()
        .expect("window lock poisoned")
        .clone();
    weak.and_then(|window| window.upgrade())
}

fn edit_dialog(deps: &impl AccessState) -> Option<crate::EditSavedDialogWindow> {
    let weak = deps
        .state()
        .edit_saved_dialog_window
        .lock()
        .expect("window lock poisoned")
        .clone();
    weak.and_then(|window| window.upgrade())
}

fn history_model(deps: &impl ClipboardService) -> ModelRc<crate::HistoryEntry> {
    let rows: Vec<_> = deps
        .history_items()
        .into_iter()
        .map(|item| {
            let normalized = item.content.replace(['\r', '\n'], " ");
            let display = if normalized.chars().count() > 120 {
                format!("{}…", normalized.chars().take(120).collect::<String>())
            } else {
                normalized
            };
            crate::HistoryEntry {
                text: display.into(),
                used: item.used,
            }
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

fn saved_items_model(deps: &impl ClipboardService) -> ModelRc<crate::SavedEntry> {
    let rows: Vec<_> = deps
        .saved_items()
        .into_iter()
        .map(|item| crate::SavedEntry {
            title: item.title.into(),
            content: item.content.into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

fn group_names_model(deps: &impl ClipboardService) -> ModelRc<SharedString> {
    let rows: Vec<SharedString> = deps.group_names().into_iter().map(Into::into).collect();
    ModelRc::new(VecModel::from(rows))
}

fn set_app_icon(window: &slint::Window) {
    let rgba = include_bytes!("../../../assets/tray-icon.rgba").to_vec();
    if let Ok(icon) = winit::window::Icon::from_rgba(rgba, 32, 32) {
        window.with_winit_window(|winit_window: &winit::window::Window| {
            winit_window.set_window_icon(Some(icon));
        });
    }
}

fn bring_to_front(window: &slint::Window) {
    window.with_winit_window(|winit_window: &winit::window::Window| {
        winit_window.focus_window();
        winit_window.request_user_attention(Some(winit::window::UserAttentionType::Informational));
    });
}

fn generate_title_from_content(content: &str) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let truncated: String = first_line.chars().take(30).collect();
    if truncated.chars().count() < first_line.chars().count() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn attach_windows(
    deps: &impl AccessState,
    history_window: &crate::HistoryWindow,
    settings_window: &crate::SettingsWindow,
    save_dialog_window: &crate::SaveDialogWindow,
    edit_saved_dialog_window: &crate::EditSavedDialogWindow,
) {
    {
        let mut history = deps
            .state()
            .history_window
            .lock()
            .expect("history window lock poisoned");
        *history = Some(history_window.as_weak());
    }
    {
        let mut settings = deps
            .state()
            .settings_window
            .lock()
            .expect("settings window lock poisoned");
        *settings = Some(settings_window.as_weak());
    }
    {
        let mut save_dialog = deps
            .state()
            .save_dialog_window
            .lock()
            .expect("save dialog window lock poisoned");
        *save_dialog = Some(save_dialog_window.as_weak());
    }
    {
        let mut edit_saved_dialog = deps
            .state()
            .edit_saved_dialog_window
            .lock()
            .expect("edit saved dialog lock poisoned");
        *edit_saved_dialog = Some(edit_saved_dialog_window.as_weak());
    }
}

fn connect_callbacks(
    send_request: RequestSender,
    history_window: &crate::HistoryWindow,
    settings_window: &crate::SettingsWindow,
    save_dialog: &crate::SaveDialogWindow,
    edit_dialog: &crate::EditSavedDialogWindow,
) {
    history_window.on_request_hide_window({
        let send_request = send_request.clone();
        move || send_request(Request::HideHistory)
    });
    history_window.on_request_select_history_item({
        let send_request = send_request.clone();
        move |index| send_request(Request::PasteHistory { index })
    });
    history_window.on_request_save_history_item({
        let send_request = send_request.clone();
        move |index| send_request(Request::OpenSaveDialog { index })
    });
    history_window.on_request_move_to_front({
        let send_request = send_request.clone();
        move |index| send_request(Request::MoveHistoryToFront { index })
    });
    history_window.on_request_bulk_paste_concat({
        let send_request = send_request.clone();
        move |index| send_request(Request::BulkPasteConcat { index })
    });
    history_window.on_request_bulk_paste_tab({
        let send_request = send_request.clone();
        move |index| send_request(Request::BulkPasteTab { index })
    });
    history_window.on_request_bulk_paste_newline({
        let send_request = send_request.clone();
        move |index| send_request(Request::BulkPasteNewline { index })
    });
    history_window.on_request_edit_saved_item({
        let send_request = send_request.clone();
        move |index| send_request(Request::OpenEditDialog { index })
    });
    history_window.on_request_switch_group({
        let send_request = send_request.clone();
        move |group_index| send_request(Request::SwitchGroup { group_index })
    });
    history_window.on_request_select_saved_item({
        let send_request = send_request.clone();
        move |index| send_request(Request::PasteSaved { index })
    });
    history_window.on_request_delete_saved_item({
        let send_request = send_request.clone();
        move |index| send_request(Request::DeleteSaved { index })
    });
    settings_window.on_request_set_hotkey_mode({
        let send_request = send_request.clone();
        move |mode| send_request(Request::SetHotkeyMode { mode })
    });
    settings_window.on_request_set_hotkey_combo_ctrl_required({
        let send_request = send_request.clone();
        move |enabled| send_request(Request::SetComboCtrl { enabled })
    });
    settings_window.on_request_set_hotkey_combo_shift_required({
        let send_request = send_request.clone();
        move |enabled| send_request(Request::SetComboShift { enabled })
    });
    settings_window.on_request_set_hotkey_combo_key({
        let send_request = send_request.clone();
        move |value| {
            send_request(Request::SetComboKey {
                value: value.to_string(),
            })
        }
    });
    save_dialog.on_request_confirm_save({
        let send_request = send_request.clone();
        move |group, title, content| {
            send_request(Request::SaveItem {
                group: group.to_string(),
                title: title.to_string(),
                content: content.to_string(),
            })
        }
    });
    save_dialog.on_request_cancel_save({
        let send_request = send_request.clone();
        move || send_request(Request::HideSaveDialog)
    });
    edit_dialog.on_request_confirm_edit({
        let send_request = send_request.clone();
        move |index, title, content| {
            send_request(Request::EditSavedItem {
                index,
                title: title.to_string(),
                content: content.to_string(),
            })
        }
    });
    edit_dialog.on_request_cancel_edit({
        let send_request = send_request.clone();
        move || send_request(Request::HideEditDialog)
    });
    history_window.window().on_close_requested({
        let send_request = send_request.clone();
        move || {
            send_request(Request::HideHistory);
            CloseRequestResponse::KeepWindowShown
        }
    });
    history_window.window().on_winit_window_event({
        let send_request = send_request.clone();
        move |_, event| {
            if let winit::event::WindowEvent::Focused(false) = event {
                send_request(Request::HideHistory);
            }
            EventResult::Propagate
        }
    });
    settings_window.window().on_close_requested({
        let send_request = send_request.clone();
        move || {
            send_request(Request::HideSettings);
            CloseRequestResponse::KeepWindowShown
        }
    });
    settings_window.window().on_winit_window_event({
        let send_request = send_request.clone();
        move |_, event| {
            if let winit::event::WindowEvent::Focused(false) = event {
                send_request(Request::HideSettings);
            }
            EventResult::Propagate
        }
    });
    save_dialog.window().on_close_requested({
        let send_request = send_request.clone();
        move || {
            send_request(Request::HideSaveDialog);
            CloseRequestResponse::KeepWindowShown
        }
    });
    edit_dialog.window().on_close_requested({
        let send_request = send_request.clone();
        move || {
            send_request(Request::HideEditDialog);
            CloseRequestResponse::KeepWindowShown
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
    use std::{
        rc::Rc,
        sync::{Arc, Mutex},
    };

    struct HeadlessPlatform;

    impl slint::platform::Platform for HeadlessPlatform {
        fn create_window_adapter(
            &self,
        ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
            Ok(MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer))
        }
    }

    #[test]
    fn ui_session_connects_requests_and_releases_windows_and_callbacks() {
        slint::platform::set_platform(Box::new(HeadlessPlatform)).unwrap();
        let app = entrait::Impl::new(App::new());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let callback_lifetime = Arc::new(());
        let weak_callback = Arc::downgrade(&callback_lifetime);
        let send_request = {
            let requests = requests.clone();
            Arc::new(move |request| {
                let _keep_alive = &callback_lifetime;
                requests.lock().unwrap().push(request);
            })
        };
        let session = app.initialize_ui(send_request).unwrap();
        assert!(history_window(&app).is_some());
        assert!(settings_window(&app).is_some());
        assert!(save_dialog(&app).is_some());
        assert!(edit_dialog(&app).is_some());

        session._history.invoke_request_select_history_item(3);
        session._settings.invoke_request_set_hotkey_mode(1);
        session._save.invoke_request_confirm_save(
            "グループ2".into(),
            "title".into(),
            "content".into(),
        );
        session
            ._edit
            .invoke_request_confirm_edit(2, "edited".into(), "updated".into());
        let requests = requests.lock().unwrap();
        match requests.as_slice() {
            [Request::PasteHistory { index: 3 }, Request::SetHotkeyMode { mode: 1 }, Request::SaveItem {
                group,
                title,
                content,
            }, Request::EditSavedItem {
                index: 2,
                title: edited_title,
                content: edited_content,
            }] => {
                assert_eq!(group, "グループ2");
                assert_eq!(title, "title");
                assert_eq!(content, "content");
                assert_eq!(edited_title, "edited");
                assert_eq!(edited_content, "updated");
            }
            _ => panic!("unexpected UI requests"),
        }
        drop(requests);
        assert!(weak_callback.upgrade().is_some());

        drop(session);
        assert!(history_window(&app).is_none());
        assert!(settings_window(&app).is_none());
        assert!(save_dialog(&app).is_none());
        assert!(edit_dialog(&app).is_none());
        assert!(weak_callback.upgrade().is_none());
    }
}

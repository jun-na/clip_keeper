use super::*;
use crate::app::domain::{
    clipboard::{HistoryItem, SavedItem},
    hotkey::KeyEvent,
    settings::HotkeySettings,
};
use std::{io, sync::Mutex, time::Instant};

#[derive(Debug, PartialEq)]
enum Call {
    PrepareHistory(i32),
    PrepareSaved(i32),
    PrepareBulk(i32, String),
    HideHistory,
    Paste,
    Observe(String),
    RefreshHistory(Option<i32>),
    Save(String, String, String),
    HideSave,
    RefreshGroups(bool),
    Edit(i32, String, String),
    HideEdit,
    RefreshSaved,
    Hotkey,
    Log(String),
    ShowHistory,
}

struct FakeServices {
    accepted: bool,
    calls: Mutex<Vec<Call>>,
}

impl FakeServices {
    fn new(accepted: bool) -> Arc<Self> {
        Arc::new(Self {
            accepted,
            calls: Mutex::new(Vec::new()),
        })
    }

    fn record(&self, call: Call) {
        self.calls.lock().unwrap().push(call);
    }
}

// 未使用の依存を呼んだ場合もテストを失敗させる。
macro_rules! unused_methods {
    ($(fn $name:ident(&self $(, $arg:ident: $ty:ty)*) $(-> $ret:ty)?;)*) => {
        $(fn $name(&self $(, $arg: $ty)*) $(-> $ret)? {
            $(let _ = $arg;)*
            panic!("unexpected call: {}", stringify!($name))
        })*
    };
}

impl ClipboardService for FakeServices {
    fn prepare_paste_from_history_index(&self, index: i32) -> bool {
        self.record(Call::PrepareHistory(index));
        self.accepted
    }
    fn prepare_paste_from_saved_index(&self, index: i32) -> bool {
        self.record(Call::PrepareSaved(index));
        self.accepted
    }
    fn prepare_bulk_paste(&self, index: i32, separator: &str) -> bool {
        self.record(Call::PrepareBulk(index, separator.into()));
        self.accepted
    }
    fn trigger_pending_paste(&self) {
        self.record(Call::Paste);
    }
    fn push_clipboard_text(&self, text: String) -> bool {
        self.record(Call::Observe(text));
        self.accepted
    }
    fn add_saved_item(&self, group: &str, title: String, content: String) {
        self.record(Call::Save(group.into(), title, content));
    }
    fn update_saved_item(&self, index: i32, title: String, content: String) {
        self.record(Call::Edit(index, title, content));
    }
    unused_methods! {
        fn history_items(&self) -> Vec<HistoryItem>;
        fn load_history_from_disk(&self) -> io::Result<()>;
        fn selected_index(&self) -> i32;
        fn get_history_item_content(&self, index: i32) -> Option<String>;
        fn remove_saved_item(&self, index: i32);
        fn saved_items(&self) -> Vec<SavedItem>;
        fn group_names(&self) -> Vec<String>;
        fn active_group_index(&self) -> i32;
        fn set_active_group(&self, group: String);
        fn move_history_to_front(&self, index: i32);
        fn get_saved_item(&self, index: i32) -> Option<(String, String)>;
    }
}

impl SettingsService for FakeServices {
    fn handle_hotkey_event(&self, _event: KeyEvent, _at: Instant) -> Option<String> {
        self.record(Call::Hotkey);
        self.accepted.then(|| "triggered".into())
    }
    unused_methods! {
        fn current_hotkey_settings(&self) -> HotkeySettings;
        fn load_from_disk(&self) -> io::Result<()>;
        fn set_hotkey_mode(&self, mode: i32);
        fn set_combo_ctrl_required(&self, enabled: bool);
        fn set_combo_shift_required(&self, enabled: bool);
        fn set_combo_key(&self, key: String);
    }
}

impl UiService for FakeServices {
    fn show_history_window(&self) {
        self.record(Call::ShowHistory);
    }
    fn hide_history_window(&self) {
        self.record(Call::HideHistory);
    }
    fn refresh_history(&self, selected_index: Option<i32>) {
        self.record(Call::RefreshHistory(selected_index));
    }
    fn hide_save_dialog(&self) {
        self.record(Call::HideSave);
    }
    fn refresh_saved_groups(&self, reset_selection: bool) {
        self.record(Call::RefreshGroups(reset_selection));
    }
    fn hide_edit_dialog(&self) {
        self.record(Call::HideEdit);
    }
    fn refresh_saved_items(&self) {
        self.record(Call::RefreshSaved);
    }
    unused_methods! {
        fn initialize_ui(&self, send_request: crate::app::request::RequestSender) -> Result<crate::app::services::ui::UiSession, slint::PlatformError>;
        fn hide_settings_window(&self);
        fn show_settings_window(&self);
        fn show_save_dialog(&self, content: &str);
        fn show_edit_dialog(&self, index: i32, title: &str, content: &str);
        fn quit_ui(&self);
    }
}

#[test]
fn paste_hides_history_before_triggering_and_stops_on_failure() {
    for accepted in [true, false] {
        for (request, preparation) in [
            (Request::PasteHistory { index: 2 }, Call::PrepareHistory(2)),
            (Request::PasteSaved { index: 3 }, Call::PrepareSaved(3)),
            (
                Request::BulkPasteConcat { index: 4 },
                Call::PrepareBulk(4, "".into()),
            ),
            (
                Request::BulkPasteTab { index: 4 },
                Call::PrepareBulk(4, "\t".into()),
            ),
            (
                Request::BulkPasteNewline { index: 4 },
                Call::PrepareBulk(4, "\n".into()),
            ),
        ] {
            let deps = FakeServices::new(accepted);
            Controller::new(deps.clone()).dispatch(request);
            let mut expected = vec![preparation];
            if accepted {
                expected.extend([Call::HideHistory, Call::Paste]);
            }
            assert_eq!(*deps.calls.lock().unwrap(), expected);
        }
    }
}

#[test]
fn clipboard_observation_refreshes_only_when_history_changes() {
    for changed in [true, false] {
        let deps = FakeServices::new(changed);
        Controller::new(deps.clone()).dispatch(Request::ClipboardObserved {
            text: "observed".into(),
        });
        let mut expected = vec![Call::Observe("observed".into())];
        if changed {
            expected.push(Call::RefreshHistory(None));
        }
        assert_eq!(*deps.calls.lock().unwrap(), expected);
    }
}

#[test]
fn saving_and_editing_update_data_before_closing_and_refreshing() {
    let deps = FakeServices::new(true);
    let controller = Controller::new(deps.clone());
    controller.dispatch(Request::SaveItem {
        group: "グループ2".into(),
        title: "title".into(),
        content: "content".into(),
    });
    controller.dispatch(Request::EditSavedItem {
        index: 1,
        title: "edited".into(),
        content: "updated".into(),
    });
    assert_eq!(
        *deps.calls.lock().unwrap(),
        vec![
            Call::Save("グループ2".into(), "title".into(), "content".into()),
            Call::HideSave,
            Call::RefreshGroups(false),
            Call::Edit(1, "edited".into(), "updated".into()),
            Call::HideEdit,
            Call::RefreshSaved,
        ]
    );
}

impl LoggingService for FakeServices {
    fn log_hotkey(&self, trigger: &str) {
        self.record(Call::Log(trigger.into()));
    }
    unused_methods! {
        fn set_hotkey_logging_enabled(&self, enabled: bool);
    }
}

#[test]
fn detected_hotkey_logs_before_showing_history() {
    use crate::app::domain::hotkey::HotkeyKey;
    for detected in [true, false] {
        let deps = FakeServices::new(detected);
        Controller::new(deps.clone()).dispatch(Request::HotkeyEvent {
            event: KeyEvent::KeyRelease(HotkeyKey::ControlLeft),
            at: Instant::now(),
        });
        let mut expected = vec![Call::Hotkey];
        if detected {
            expected.extend([Call::Log("triggered".into()), Call::ShowHistory]);
        }
        assert_eq!(*deps.calls.lock().unwrap(), expected);
    }
}

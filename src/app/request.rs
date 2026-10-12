use crate::app::domain::hotkey::KeyEvent;
use std::sync::Arc;
use std::time::Instant;

/// 外部入力をアプリのRequest受付へ渡す依存。
pub type RequestSender = Arc<dyn Fn(Request) + Send + Sync>;

/// Requests from UI, tray and background monitors.
pub enum Request {
    HideHistory,
    PasteHistory {
        index: i32,
    },
    OpenSaveDialog {
        index: i32,
    },
    MoveHistoryToFront {
        index: i32,
    },
    BulkPasteConcat {
        index: i32,
    },
    BulkPasteTab {
        index: i32,
    },
    BulkPasteNewline {
        index: i32,
    },
    OpenEditDialog {
        index: i32,
    },
    SwitchGroup {
        group_index: i32,
    },
    PasteSaved {
        index: i32,
    },
    DeleteSaved {
        index: i32,
    },
    SetHotkeyMode {
        mode: i32,
    },
    SetComboCtrl {
        enabled: bool,
    },
    SetComboShift {
        enabled: bool,
    },
    SetComboKey {
        value: String,
    },
    SaveItem {
        group: String,
        title: String,
        content: String,
    },
    HideSaveDialog,
    EditSavedItem {
        index: i32,
        title: String,
        content: String,
    },
    HideEditDialog,
    ClipboardObserved {
        text: String,
    },
    HotkeyEvent {
        event: KeyEvent,
        at: Instant,
    },
    ShowHistory,
    ShowSettings,
    HideSettings,
    Quit,
}

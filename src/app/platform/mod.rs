// OS ごとの実装差分を閉じ込める。
pub mod hotkey_event;
#[cfg(target_os = "macos")]
pub mod macos_hotkey_listener;

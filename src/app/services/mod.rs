// ビジネスロジック／UI連携ロジックを提供するサービス群。
pub mod clipboard_service;
pub mod detectors;
pub mod hotkey_logger;
#[cfg(target_os = "macos")]
pub mod macos_hotkey_listener;
pub mod monitor_runtime;
pub mod settings_service;
pub mod tray_runtime;
pub mod ui_gateway;

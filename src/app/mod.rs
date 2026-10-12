// アプリケーション層のルート。共有状態とentraitの依存を組み立てる。
pub mod domain;
pub mod platform;
pub mod runtime;
pub mod services;
pub mod state;
pub mod storage_paths;

pub mod controller;
pub mod mediator;
pub mod request;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use entrait::Impl;

use runtime::application_runtime::ApplicationRuntime;
use services::{clipboard::ClipboardService, settings::SettingsService};
use state::AppState;

/// ステートレスなサービス関数が参照する共有依存。
pub struct App {
    state: Arc<AppState>,
    hotkey_log_path: PathBuf,
}

impl App {
    fn new() -> Self {
        Self {
            state: Arc::new(AppState::new()),
            hotkey_log_path: storage_paths::data_file_path("hotkey_log.txt")
                .unwrap_or_else(|_| PathBuf::from("hotkey_log.txt")),
        }
    }
}

/// サービスから共有状態を取得する境界。
#[entrait::entrait]
pub trait AccessState {
    fn state(&self) -> &AppState;
}

impl AccessState for App {
    fn state(&self) -> &AppState {
        &self.state
    }
}

/// ログサービスが使用する出力先。
#[entrait::entrait]
pub trait AccessLogPath {
    fn hotkey_log_path(&self) -> &Path;
}

impl AccessLogPath for App {
    fn hotkey_log_path(&self) -> &Path {
        &self.hotkey_log_path
    }
}

pub type Application = Impl<App>;

/// 状態の生成・復元と実行系の依存解決を集約するComposition Root。
pub fn build() -> Arc<ApplicationRuntime> {
    let app = Arc::new(Impl::new(App::new()));
    // 読み込み失敗時は初期状態で起動を継続。
    if let Err(error) = app.load_history_from_disk() {
        eprintln!("failed to load clipboard history: {error}");
    }
    if let Err(error) = app.load_from_disk() {
        eprintln!("failed to load settings: {error}");
    }
    Arc::new(ApplicationRuntime::new(app))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::domain::hotkey::{HotkeyKey, KeyEvent};
    use std::time::{Duration, Instant};

    #[test]
    fn entrait_services_share_the_app_state() {
        let app = Arc::new(Impl::new(App::new()));
        app.state()
            .clipboard_state
            .lock()
            .unwrap()
            .push_clipboard("shared".into());
        app.state()
            .settings_state
            .lock()
            .unwrap()
            .set_hotkey_mode(1);
        let second = app.clone();
        assert_eq!(app.history_items()[0].content, "shared");
        assert_eq!(second.current_hotkey_settings().hotkey_mode, 1);

        let at = Instant::now();
        let event = KeyEvent::KeyRelease(HotkeyKey::ControlLeft);
        assert!(app.handle_hotkey_event(event, at).is_none());
        assert!(second
            .handle_hotkey_event(event, at + Duration::from_millis(100))
            .is_some());

        app.state()
            .clipboard_state
            .lock()
            .unwrap()
            .push_clipboard("updated".into());
        assert_eq!(second.history_items()[0].content, "updated");
    }

    #[test]
    fn separate_apps_have_independent_state() {
        let first = Impl::new(App::new());
        let second = Impl::new(App::new());
        first
            .state()
            .clipboard_state
            .lock()
            .unwrap()
            .push_clipboard("first".into());
        first
            .state()
            .settings_state
            .lock()
            .unwrap()
            .set_hotkey_mode(1);
        assert!(second.history_items().is_empty());
        assert_eq!(second.current_hotkey_settings().hotkey_mode, 0);
    }

    #[test]
    fn dropping_runtime_releases_the_app() {
        let app = Arc::new(Impl::new(App::new()));
        let weak = Arc::downgrade(&app);
        let runtime = ApplicationRuntime::new(app);
        assert!(weak.upgrade().is_some());
        drop(runtime);
        assert!(weak.upgrade().is_none());
    }
}

use crate::app::runtime::application_runtime::ApplicationRuntime;
use crate::app::runtime::{monitor_runtime::MonitorRuntime, tray_runtime::TrayRuntime};
use crate::app::services::{tray::TraySession, ui::UiSession};
use std::sync::{Arc, Mutex};

// サービスのセッションと入力ランタイムの起動・寿命を管理。
pub struct ServiceRuntime {
    _ui_session: UiSession,
    _tray_session: TraySession,
    _tray_listener: TrayRuntime,
    app: Arc<ApplicationRuntime>,
    monitor_runtime: Mutex<Option<MonitorRuntime>>,
}

impl ServiceRuntime {
    pub fn new(app: Arc<ApplicationRuntime>) -> Result<Self, Box<dyn std::error::Error>> {
        let ui_session = app.initialize_ui()?;
        let tray_session = app.initialize_tray()?;
        let tray_listener = app.start_tray_listener(tray_session.menu_ids());
        Ok(Self {
            _ui_session: ui_session,
            _tray_session: tray_session,
            _tray_listener: tray_listener,
            app,
            monitor_runtime: Mutex::new(None),
        })
    }

    pub fn start_background_services(&self) {
        let mut session = self
            .monitor_runtime
            .lock()
            .expect("monitor session lock poisoned");
        if session.is_none() {
            *session = Some(self.app.start_monitors());
        }
    }
}

use crate::app::request::RequestSender;
use crate::app::runtime::{monitor_runtime::MonitorRuntime, tray_runtime::TrayRuntime};
use crate::app::services::{
    tray::{TrayMenuIds, TrayService, TraySession},
    ui::{UiService, UiSession},
};
use crate::app::{controller::Controller, mediator::Mediator, Application};
use std::sync::Arc;

// Appを参照する実行系を保持。Appから実行系への強参照は持たせない。
pub struct ApplicationRuntime {
    app: Arc<Application>,
    mediator: Arc<Mediator>,
}

impl ApplicationRuntime {
    pub fn new(app: Arc<Application>) -> Self {
        let controller = Arc::new(Controller::new(app.clone()));
        let mediator = Arc::new(Mediator::new(controller.clone()));
        Self { app, mediator }
    }

    fn request_sender(&self) -> RequestSender {
        let mediator = self.mediator.clone();
        Arc::new(move |request| mediator.send(request))
    }

    pub(crate) fn initialize_ui(&self) -> Result<UiSession, slint::PlatformError> {
        self.app.initialize_ui(self.request_sender())
    }

    pub(crate) fn initialize_tray(&self) -> Result<TraySession, Box<dyn std::error::Error>> {
        self.app.initialize_tray()
    }

    pub(crate) fn start_tray_listener(&self, ids: TrayMenuIds) -> TrayRuntime {
        TrayRuntime::start(ids, self.request_sender())
    }

    pub(crate) fn start_monitors(&self) -> MonitorRuntime {
        MonitorRuntime::start(self.request_sender())
    }
}

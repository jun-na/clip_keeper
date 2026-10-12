use crate::app::request::{Request, RequestSender};
use crate::app::services::tray::TrayMenuIds;
use std::thread;
use tray_icon::menu::MenuEvent;

/// トレイのイベント待ちとRequest送信を担当。
pub struct TrayRuntime {
    _listener: thread::JoinHandle<()>,
}

impl TrayRuntime {
    pub fn start(ids: TrayMenuIds, send_request: RequestSender) -> Self {
        let thread = thread::spawn(move || {
            while let Ok(event) = MenuEvent::receiver().recv() {
                if event.id == ids.open_history {
                    send_request(Request::ShowHistory);
                } else if event.id == ids.open_settings {
                    send_request(Request::ShowSettings);
                } else if event.id == ids.quit {
                    send_request(Request::Quit);
                    break;
                }
            }
        });
        Self { _listener: thread }
    }
}

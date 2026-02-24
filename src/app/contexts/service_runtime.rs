use crate::app::contexts::service_context::ServiceContext;
use crate::app::services::monitor_runtime::MonitorRuntime;
use crate::app::services::tray_runtime::TrayRuntime;

// 実行中のサービス実体を保持するランタイム。
pub struct ServiceRuntime {
    // TrayIcon は drop すると消えるため、ランタイムで保持する。
    _tray_runtime: TrayRuntime,
    monitor_runtime: MonitorRuntime,
}

impl ServiceRuntime {
    /// 実行系サービスを生成し、Window と接続する。
    pub fn new(
        service_context: &ServiceContext,
        history_window: &crate::HistoryWindow,
        settings_window: &crate::SettingsWindow,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ui_gateway = service_context.ui_gateway();
        ui_gateway.attach_windows(history_window, settings_window);

        // TrayRuntime は UiGateway を使ってメニュー操作をUIへ橋渡しする。
        let tray_runtime = TrayRuntime::new(ui_gateway.clone())?;
        // MonitorRuntime は ClipboardService / SettingsService / UiGateway を使って監視を行う。
        let monitor_runtime = MonitorRuntime::new(
            service_context.clipboard_service(),
            service_context.settings_service(),
            ui_gateway,
        );

        Ok(Self {
            _tray_runtime: tray_runtime,
            monitor_runtime,
        })
    }

    /// バックグラウンドで動く監視系サービスを開始する。
    pub fn start_background_services(&self) {
        self.monitor_runtime.start();
    }
}

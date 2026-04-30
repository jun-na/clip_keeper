use crate::app::contexts::state_context::StateContext;
use crate::app::services::app_activation_service::AppActivationService;
use crate::app::services::clipboard_service::ClipboardService;
use crate::app::services::hotkey_service::HotkeyService;
use crate::app::services::settings_service::SettingsService;
use crate::app::services::ui_gateway::UiGateway;
use std::sync::Arc;

// アプリ全体で共有する「状態」と「サービス実体」を束ねるコンテキスト。
pub struct AppContext {
    pub(crate) _state_context: Arc<StateContext>,
    pub(crate) _app_activation_service: Arc<AppActivationService>,
    pub(crate) clipboard_service: Arc<ClipboardService>,
    pub(crate) settings_service: Arc<SettingsService>,
    pub(crate) hotkey_service: Arc<HotkeyService>,
    pub(crate) ui_gateway: Arc<UiGateway>,
}

impl AppContext {
    /// State と Service を束ねた AppContext を生成する。
    /// 依存を組み立てた `Self` を `Result` で返す。
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let state_context = StateContext::new();
        let app_activation_service = Arc::new(AppActivationService::new());
        let clipboard_service = Arc::new(ClipboardService::new(
            state_context.clone(),
            app_activation_service.clone(),
        ));
        let settings_service = Arc::new(SettingsService::new(state_context.clone()));
        let hotkey_service = Arc::new(HotkeyService::new(settings_service.clone()));
        let ui_gateway = Arc::new(UiGateway::new(
            clipboard_service.clone(),
            settings_service.clone(),
        ));

        Ok(Self {
            _state_context: state_context,
            _app_activation_service: app_activation_service,
            clipboard_service,
            settings_service,
            hotkey_service,
            ui_gateway,
        })
    }
}

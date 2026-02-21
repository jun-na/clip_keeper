use std::sync::Arc;

use crate::app::contexts::state_context::StateContext;
use crate::app::services::clipboard_service::ClipboardService;
use crate::app::services::ui_gateway::UiGateway;

// サービスの定義と依存関係を保持するコンテキスト。
pub struct ServiceContext {
    clipboard_service: Arc<ClipboardService>,
    ui_gateway: Arc<UiGateway>,
}

impl ServiceContext {
    /// サービス定義を構築する。
    /// ここでは実行開始はせず、依存関係だけを解決する。
    pub fn new(state_context: Arc<StateContext>) -> Result<Self, Box<dyn std::error::Error>> {
        // 状態アクセス専用サービス。
        let clipboard_service = Arc::new(ClipboardService::new(state_context));
        // UI操作サービス。ClipboardService を使って表示データを取得する。
        let ui_gateway = Arc::new(UiGateway::new(clipboard_service.clone()));

        Ok(Self {
            clipboard_service,
            ui_gateway,
        })
    }

    /// クリップボード関連サービスを取得する。
    pub(crate) fn clipboard_service(&self) -> Arc<ClipboardService> {
        self.clipboard_service.clone()
    }

    /// UI操作サービスを取得する。
    pub(crate) fn ui_gateway(&self) -> Arc<UiGateway> {
        self.ui_gateway.clone()
    }
}

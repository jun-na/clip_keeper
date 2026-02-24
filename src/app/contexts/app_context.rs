use crate::app::contexts::service_context::ServiceContext;
use crate::app::contexts::state_context::StateContext;

// アプリ全体で共有する「状態」と「サービス定義」を束ねるコンテキスト。
pub struct AppContext {
    _state_context: std::sync::Arc<StateContext>,
    service_context: ServiceContext,
}

impl AppContext {
    /// StateContext と ServiceContext を受け取り AppContext を生成する。
    pub fn new(
        state_context: std::sync::Arc<StateContext>,
        service_context: ServiceContext,
    ) -> Self {
        Self {
            _state_context: state_context,
            service_context,
        }
    }

    /// サービス定義コンテキストを参照する。
    pub fn service_context(&self) -> &ServiceContext {
        &self.service_context
    }
}

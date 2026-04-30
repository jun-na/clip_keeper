use crate::app::contexts::app_context::AppContext;
use crate::app::contexts::service_context::ServiceContext;
use crate::app::contexts::state_context::StateContext;

// 依存関係を1箇所で組み立てる Composition Root。
pub struct CompositionRoot;

impl CompositionRoot {
    /// アプリで必要な Context 群を組み立てる。
    pub fn build() -> Result<AppContext, Box<dyn std::error::Error>> {
        // 共有状態を先に作り、サービス群へ注入する。
        let state_context = StateContext::new();
        let service_context = ServiceContext::new(state_context.clone())?;

        Ok(AppContext::new(state_context, service_context))
    }
}

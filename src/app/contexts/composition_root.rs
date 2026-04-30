use crate::app::contexts::app_context::AppContext;
use entrait::Impl;

// 依存関係を1箇所で組み立てる Composition Root。
pub struct CompositionRoot;

impl CompositionRoot {
    /// アプリで必要な Context 群を組み立てる。
    /// `Impl<AppContext>` を返し、DI の起点になる。
    pub fn build() -> Result<Impl<AppContext>, Box<dyn std::error::Error>> {
        Ok(Impl::new(AppContext::new()?))
    }
}

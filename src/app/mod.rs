// アプリケーション層のルートモジュール。
// feature を中心に公開し、旧来の service/state は内部実装へ閉じ込める。
pub mod contexts;
pub mod features;
pub mod platform;
pub mod runtime;
pub mod storage_paths;

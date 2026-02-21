#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// エントリーポイント。
// 依存解決・Window生成・サービス起動・UIイベントループ開始までを担当する。

use slint::ComponentHandle;
use std::error::Error;

slint::include_modules!();

mod app;

/// アプリ起動シーケンスを実行する。
fn main() -> Result<(), Box<dyn Error>> {
    // 依存関係（状態/サービス定義）を組み立てる。
    let app = app::contexts::composition_root::CompositionRoot::build()?;

    // UI本体を生成。実行中は main スコープで保持して寿命を維持する。
    let history_window = HistoryWindow::new()?;
    history_window.hide()?;

    let settings_window = SettingsWindow::new()?;
    settings_window.hide()?;

    // 実行系サービス（トレイ/監視）を起動。
    let service_runtime = app::contexts::service_runtime::ServiceRuntime::new(
        app.service_context(),
        &history_window,
        &settings_window,
    )?;
    service_runtime.start_background_services();

    // Slint のメインイベントループ（ここでアプリが待機する）。
    slint::run_event_loop()?;
    Ok(())
}

#![windows_subsystem = "windows"]
// エントリーポイント。
// 依存解決・Window生成・サービス起動・UIイベントループ開始までを担当する。

#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use slint::ComponentHandle;
use std::error::Error;
use std::sync::Arc;

slint::include_modules!();

mod app;

use crate::app::services::clipboard_service::ClipboardServiceApi;
use crate::app::services::settings_service::SettingsServiceApi;

/// アプリ起動シーケンスを実行する。
/// 正常終了時は `Ok(())` を返し、UI と常駐サービスを起動する。
fn main() -> Result<(), Box<dyn Error>> {
    // DragWindow / Focusイベントを使うため winit backend を明示選択する。
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    #[cfg(target_os = "macos")]
    configure_macos_activation_policy();

    // 依存関係（状態/サービス定義）を組み立てる。
    let app = Arc::new(app::contexts::composition_root::CompositionRoot::build()?);
    // 前回終了時の履歴をロードする（読み込み失敗は継続可能）。
    if let Err(error) = app.load_history_from_disk() {
        eprintln!("failed to load clipboard history: {error}");
    }
    // 前回終了時の設定をロードする（読み込み失敗は継続可能）。
    if let Err(error) = app.load_from_disk() {
        eprintln!("failed to load settings: {error}");
    }

    // UI本体を生成。実行中は main スコープで保持して寿命を維持する。
    let history_window = HistoryWindow::new()?;
    history_window.hide()?;

    let settings_window = SettingsWindow::new()?;
    settings_window.hide()?;

    let save_dialog_window = SaveDialogWindow::new()?;
    save_dialog_window.hide()?;

    let edit_saved_dialog_window = EditSavedDialogWindow::new()?;
    edit_saved_dialog_window.hide()?;

    // 実行系サービス（トレイ/監視）を起動。
    let service_runtime = app::contexts::service_runtime::ServiceRuntime::new(
        app.clone(),
        &history_window,
        &settings_window,
        &save_dialog_window,
        &edit_saved_dialog_window,
    )?;
    service_runtime.start_background_services();

    // トレイ常駐アプリのため、全ウィンドウ非表示でも終了しないイベントループを使う。
    slint::run_event_loop_until_quit()?;
    Ok(())
}

#[cfg(target_os = "macos")]
/// macOS のアクティベーションポリシーを常駐向けに設定する。
/// Dock 表示を抑え、アクセサリ扱いでアプリを動かす。
fn configure_macos_activation_policy() {
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let app = NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }
}

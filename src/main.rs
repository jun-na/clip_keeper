#![windows_subsystem = "windows"]
// エントリーポイント。
// 依存解決・Window生成・サービス起動・UIイベントループ開始までを担当する。

use slint::ComponentHandle;
use std::error::Error;

slint::include_modules!();

mod app;

/// アプリ起動シーケンスを実行する。
fn main() -> Result<(), Box<dyn Error>> {
    // DragWindow / Focusイベントを使うため winit backend を明示選択する。
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    // 依存関係（状態/サービス定義）を組み立てる。
    let app = app::contexts::composition_root::CompositionRoot::build()?;
    // 前回終了時の履歴をロードする（読み込み失敗は継続可能）。
    if let Err(error) = app
        .service_context()
        .clipboard_service()
        .load_history_from_disk()
    {
        eprintln!("failed to load clipboard history: {error}");
    }
    // 前回終了時の設定をロードする（読み込み失敗は継続可能）。
    if let Err(error) = app.service_context().settings_service().load_from_disk() {
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

    let group_name_dialog_window = GroupNameDialogWindow::new()?;
    group_name_dialog_window.hide()?;

    let group_delete_dialog_window = GroupDeleteDialogWindow::new()?;
    group_delete_dialog_window.hide()?;

    // 実行系サービス（トレイ/監視）を起動。
    let service_runtime = app::contexts::service_runtime::ServiceRuntime::new(
        app.service_context(),
        &history_window,
        &settings_window,
        &save_dialog_window,
        &edit_saved_dialog_window,
        &group_name_dialog_window,
        &group_delete_dialog_window,
    )?;
    service_runtime.start_background_services();

    // トレイ常駐アプリのため、全ウィンドウ非表示でも終了しないイベントループを使う。
    slint::run_event_loop_until_quit()?;
    Ok(())
}

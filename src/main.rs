#![windows_subsystem = "windows"]
// エントリーポイント。
// 依存解決・サービス起動・UIイベントループ開始までを担当する。

#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use std::error::Error;

slint::include_modules!();

mod app;

/// アプリ起動シーケンスを実行する。
fn main() -> Result<(), Box<dyn Error>> {
    // DragWindow / Focusイベントを使うため winit backend を明示選択する。
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .select()?;

    #[cfg(target_os = "macos")]
    configure_macos_activation_policy();

    // 共有状態を生成・復元し、entraitの実装と実行系を組み立てる。
    let app = app::build();

    // UI・トレイ・監視を初期化。画面の寿命はServiceRuntimeで保持する。
    let service_runtime = app::runtime::service_runtime::ServiceRuntime::new(app.clone())?;
    service_runtime.start_background_services();

    // トレイ常駐アプリのため、全ウィンドウ非表示でも終了しないイベントループを使う。
    slint::run_event_loop_until_quit()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_macos_activation_policy() {
    let main_thread = MainThreadMarker::new().expect("application must start on the main thread");
    let app = NSApplication::sharedApplication(main_thread);
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

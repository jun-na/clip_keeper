use std::sync::Arc;
use std::thread;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::services::ui_gateway::UiGateway;

// タスクトレイとメニュー操作を扱うサービス。
pub struct TrayRuntime {
    // TrayIcon は drop で消えるため保持が必要。
    _tray_icon: TrayIcon,
}

impl TrayRuntime {
    /// タスクトレイを生成し、メニューイベント監視を開始する。
    pub fn new(ui_gateway: Arc<UiGateway>) -> Result<Self, Box<dyn std::error::Error>> {
        let tray_menu = Menu::new();
        let open_history_item = MenuItem::new("Open History", true, None);
        let open_settings_item = MenuItem::new("Settings", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        tray_menu.append(&open_history_item)?;
        tray_menu.append(&open_settings_item)?;
        tray_menu.append(&quit_item)?;

        let tray_icon = Self::create_tray_icon(&tray_menu)?;

        Self::start_listener(
            open_history_item.id().clone(),
            open_settings_item.id().clone(),
            quit_item.id().clone(),
            ui_gateway,
        );

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }

    /// 単色アイコン付きの TrayIcon を作成する。
    fn create_tray_icon(menu: &Menu) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        let mut rgba = Vec::with_capacity(32 * 32 * 4);
        for _ in 0..(32 * 32) {
            rgba.extend_from_slice(&[0x0F, 0x7D, 0xD6, 0xFF]);
        }

        let icon = Icon::from_rgba(rgba, 32, 32)?;

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Clip Keeper")
            .with_icon(icon)
            .with_menu(Box::new(menu.clone()))
            .build()?;

        Ok(tray_icon)
    }

    /// タスクトレイメニューのイベントループをバックグラウンドで実行する。
    fn start_listener(
        open_history_id: MenuId,
        open_settings_id: MenuId,
        quit_id: MenuId,
        ui_gateway: Arc<UiGateway>,
    ) {
        // メニューイベントを監視し、UiGateway 経由でUIへ操作を伝える。
        thread::spawn(move || {
            while let Ok(event) = MenuEvent::receiver().recv() {
                if event.id == open_history_id {
                    ui_gateway.show_history_window();
                } else if event.id == open_settings_id {
                    ui_gateway.show_settings_window();
                } else if event.id == quit_id {
                    let _ = slint::invoke_from_event_loop(|| {
                        slint::quit_event_loop().ok();
                    });
                    break;
                }
            }
        });
    }
}

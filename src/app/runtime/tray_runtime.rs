use std::sync::Arc;
use std::thread;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::runtime::ui_gateway::UiGateway;

pub struct TrayRuntime {
    _tray_icon: TrayIcon,
}

impl TrayRuntime {
    pub fn new(ui_gateway: Arc<UiGateway>) -> Result<Self, Box<dyn std::error::Error>> {
        let tray_menu = Menu::new();
        let open_history_item = MenuItem::new("履歴を開く", true, None);
        let open_settings_item = MenuItem::new("設定", true, None);
        let quit_item = MenuItem::new("終了", true, None);

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

    fn create_tray_icon(menu: &Menu) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        let rgba = include_bytes!("../../../assets/tray-icon.rgba").to_vec();

        let icon = Icon::from_rgba(rgba, 32, 32)?;

        #[cfg(target_os = "macos")]
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("クリップキーパー")
            .with_icon(icon)
            .with_icon_as_template(true)
            .with_menu(Box::new(menu.clone()))
            .build()?;

        #[cfg(not(target_os = "macos"))]
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("クリップキーパー")
            .with_icon(icon)
            .with_menu(Box::new(menu.clone()))
            .build()?;

        Ok(tray_icon)
    }

    fn start_listener(
        open_history_id: MenuId,
        open_settings_id: MenuId,
        quit_id: MenuId,
        ui_gateway: Arc<UiGateway>,
    ) {
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
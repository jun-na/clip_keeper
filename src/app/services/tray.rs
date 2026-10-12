use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct TraySession {
    _tray_icon: TrayIcon,
    menu_ids: TrayMenuIds,
}

pub(crate) struct TrayMenuIds {
    pub open_history: MenuId,
    pub open_settings: MenuId,
    pub quit: MenuId,
}

impl TraySession {
    pub(crate) fn menu_ids(&self) -> TrayMenuIds {
        TrayMenuIds {
            open_history: self.menu_ids.open_history.clone(),
            open_settings: self.menu_ids.open_settings.clone(),
            quit: self.menu_ids.quit.clone(),
        }
    }
}

#[entrait::entrait(pub TrayService)]
mod operations {
    use super::*;
    pub fn initialize_tray<D>(_deps: &D) -> Result<TraySession, Box<dyn std::error::Error>> {
        let tray_menu = Menu::new();
        let open_history_item = MenuItem::new("履歴を開く", true, None);
        let open_settings_item = MenuItem::new("設定", true, None);
        let quit_item = MenuItem::new("終了", true, None);

        tray_menu.append(&open_history_item)?;
        tray_menu.append(&open_settings_item)?;
        tray_menu.append(&quit_item)?;

        let tray_icon = create_tray_icon(&tray_menu)?;

        Ok(TraySession {
            _tray_icon: tray_icon,
            menu_ids: TrayMenuIds {
                open_history: open_history_item.id().clone(),
                open_settings: open_settings_item.id().clone(),
                quit: quit_item.id().clone(),
            },
        })
    }
}

fn create_tray_icon(menu: &Menu) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let rgba = include_bytes!("../../../assets/tray-icon.rgba").to_vec();

    let icon = Icon::from_rgba(rgba, 32, 32)?;

    #[cfg(target_os = "macos")]
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("クリップキーパー")
        .with_icon_templated(icon)
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

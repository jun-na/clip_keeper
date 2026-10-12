use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::domain::hotkey::KeyEvent;
use crate::app::domain::settings::{normalize_combo_key, HotkeySettings, SettingsState};
use crate::app::AccessState;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSettings {
    version: u32,
    #[serde(default)]
    hotkey_mode: i32,
    #[serde(default)]
    hotkey_combo_ctrl_required: bool,
    #[serde(default)]
    hotkey_combo_shift_required: bool,
    #[serde(default)]
    hotkey_combo_key: String,
    #[serde(default)]
    hotkey_ctrl_double_tap_enabled: Option<bool>,
    #[serde(default)]
    hotkey_shift_double_tap_enabled: Option<bool>,
}

// 公開関数からサービスtraitと Impl<App> の実装を生成。
#[entrait::entrait(pub SettingsService)]
mod operations {
    use super::*;

    pub fn current_hotkey_settings(deps: &impl AccessState) -> HotkeySettings {
        let state = deps
            .state()
            .settings_state
            .lock()
            .expect("settings state lock poisoned");

        state.hotkey_settings()
    }

    pub fn handle_hotkey_event(
        deps: &impl AccessState,
        event: KeyEvent,
        at: Instant,
    ) -> Option<String> {
        let settings = current_hotkey_settings(deps);
        deps.state()
            .hotkey_state
            .lock()
            .expect("hotkey state lock poisoned")
            .handle_event(event, &settings, at)
    }

    pub fn load_from_disk(deps: &impl AccessState) -> io::Result<()> {
        let path = settings_file_path()?;
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let persisted: PersistedSettings = serde_json::from_str(&content).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid settings json: {err}"),
            )
        })?;

        let mut state = deps
            .state()
            .settings_state
            .lock()
            .expect("settings state lock poisoned");

        if persisted.hotkey_mode == 0 {
            if let Some(true) = persisted.hotkey_ctrl_double_tap_enabled {
                state.hotkey_mode = 1;
            } else if let Some(true) = persisted.hotkey_shift_double_tap_enabled {
                state.hotkey_mode = 0;
            } else {
                state.hotkey_mode = persisted.hotkey_mode;
            }
        } else {
            state.hotkey_mode = persisted.hotkey_mode;
        }

        state.hotkey_combo_ctrl_required = persisted.hotkey_combo_ctrl_required;
        state.hotkey_combo_shift_required = persisted.hotkey_combo_shift_required;
        state.hotkey_combo_key = normalize_combo_key(&persisted.hotkey_combo_key);

        Ok(())
    }

    pub fn set_hotkey_mode(deps: &impl AccessState, mode: i32) {
        let mut state = deps
            .state()
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.set_hotkey_mode(mode);
        save_state_locked(&state);
    }

    pub fn set_combo_ctrl_required(deps: &impl AccessState, enabled: bool) {
        let mut state = deps
            .state()
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_combo_ctrl_required = enabled;
        save_state_locked(&state);
    }

    pub fn set_combo_shift_required(deps: &impl AccessState, enabled: bool) {
        let mut state = deps
            .state()
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_combo_shift_required = enabled;
        save_state_locked(&state);
    }

    pub fn set_combo_key(deps: &impl AccessState, key: String) {
        let mut state = deps
            .state()
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.set_combo_key(&key);
        save_state_locked(&state);
    }
}

fn save_state_locked(state: &SettingsState) {
    if let Err(error) = save_to_disk_locked(state) {
        eprintln!("failed to save settings: {error}");
    }
}

fn save_to_disk_locked(state: &SettingsState) -> io::Result<()> {
    let payload = PersistedSettings {
        version: 2,
        hotkey_mode: state.hotkey_mode,
        hotkey_combo_ctrl_required: state.hotkey_combo_ctrl_required,
        hotkey_combo_shift_required: state.hotkey_combo_shift_required,
        hotkey_combo_key: state.hotkey_combo_key.clone(),
        hotkey_ctrl_double_tap_enabled: None,
        hotkey_shift_double_tap_enabled: None,
    };

    let json = serde_json::to_string_pretty(&payload)
        .map_err(|err| io::Error::other(format!("serialize settings error: {err}")))?;
    fs::write(settings_file_path()?, json)?;
    Ok(())
}

fn settings_file_path() -> io::Result<PathBuf> {
    crate::app::storage_paths::data_file_path("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::domain::hotkey::HotkeyKey;
    use crate::app::App;
    use std::time::Duration;

    #[test]
    fn hotkey_operation_uses_shared_settings_and_detector_state() {
        let app = App::new();
        app.state()
            .settings_state
            .lock()
            .unwrap()
            .set_hotkey_mode(1);
        let service = entrait::Impl::new(app);
        let at = Instant::now();
        let event = KeyEvent::KeyRelease(HotkeyKey::ControlLeft);
        assert!(service.handle_hotkey_event(event, at).is_none());
        assert!(service
            .handle_hotkey_event(event, at + Duration::from_millis(100))
            .is_some());
    }
}

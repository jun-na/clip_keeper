use std::sync::Arc;
use std::{env, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::contexts::state_context::StateContext;
use crate::app::states::settings_state::SettingsState;

#[derive(Debug, Clone)]
pub struct HotkeySettings {
    pub ctrl_double_tap_enabled: bool,
    pub shift_double_tap_enabled: bool,
    pub combo_ctrl_required: bool,
    pub combo_shift_required: bool,
    pub combo_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSettings {
    version: u32,
    hotkey_ctrl_double_tap_enabled: bool,
    hotkey_shift_double_tap_enabled: bool,
    hotkey_combo_ctrl_required: bool,
    hotkey_combo_shift_required: bool,
    hotkey_combo_key: String,
}

// ホットキー設定の読み書き・永続化を集約するサービス。
pub struct SettingsService {
    state_context: Arc<StateContext>,
}

impl SettingsService {
    pub fn new(state_context: Arc<StateContext>) -> Self {
        Self { state_context }
    }

    pub fn current_hotkey_settings(&self) -> HotkeySettings {
        let state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");

        HotkeySettings {
            ctrl_double_tap_enabled: state.hotkey_ctrl_double_tap_enabled,
            shift_double_tap_enabled: state.hotkey_shift_double_tap_enabled,
            combo_ctrl_required: state.hotkey_combo_ctrl_required,
            combo_shift_required: state.hotkey_combo_shift_required,
            combo_key: state.hotkey_combo_key.clone(),
        }
    }

    pub fn load_from_disk(&self) -> io::Result<()> {
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

        let mut state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_ctrl_double_tap_enabled = persisted.hotkey_ctrl_double_tap_enabled;
        state.hotkey_shift_double_tap_enabled = persisted.hotkey_shift_double_tap_enabled;
        state.hotkey_combo_ctrl_required = persisted.hotkey_combo_ctrl_required;
        state.hotkey_combo_shift_required = persisted.hotkey_combo_shift_required;
        state.hotkey_combo_key = normalize_combo_key(&persisted.hotkey_combo_key);

        Ok(())
    }

    pub fn set_ctrl_double_tap_enabled(&self, enabled: bool) {
        let mut state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_ctrl_double_tap_enabled = enabled;
        self.save_state_locked(&state);
    }

    pub fn set_shift_double_tap_enabled(&self, enabled: bool) {
        let mut state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_shift_double_tap_enabled = enabled;
        self.save_state_locked(&state);
    }

    pub fn set_combo_ctrl_required(&self, enabled: bool) {
        let mut state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_combo_ctrl_required = enabled;
        self.save_state_locked(&state);
    }

    pub fn set_combo_shift_required(&self, enabled: bool) {
        let mut state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_combo_shift_required = enabled;
        self.save_state_locked(&state);
    }

    pub fn set_combo_key(&self, key: String) {
        let mut state = self
            .state_context
            .settings_state
            .lock()
            .expect("settings state lock poisoned");
        state.hotkey_combo_key = normalize_combo_key(&key);
        self.save_state_locked(&state);
    }

    fn save_state_locked(&self, state: &SettingsState) {
        if let Err(error) = self.save_to_disk_locked(state) {
            eprintln!("failed to save settings: {error}");
        }
    }

    fn save_to_disk_locked(&self, state: &SettingsState) -> io::Result<()> {
        let payload = PersistedSettings {
            version: 1,
            hotkey_ctrl_double_tap_enabled: state.hotkey_ctrl_double_tap_enabled,
            hotkey_shift_double_tap_enabled: state.hotkey_shift_double_tap_enabled,
            hotkey_combo_ctrl_required: state.hotkey_combo_ctrl_required,
            hotkey_combo_shift_required: state.hotkey_combo_shift_required,
            hotkey_combo_key: state.hotkey_combo_key.clone(),
        };

        let json = serde_json::to_string_pretty(&payload)
            .map_err(|err| io::Error::other(format!("serialize settings error: {err}")))?;
        fs::write(settings_file_path()?, json)?;
        Ok(())
    }
}

fn normalize_combo_key(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "H".to_string();
    }

    let first = trimmed.chars().next().unwrap_or('H');
    if first.is_ascii_alphanumeric() {
        first.to_ascii_uppercase().to_string()
    } else {
        "H".to_string()
    }
}

fn settings_file_path() -> io::Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve executable directory",
        )
    })?;
    Ok(exe_dir.join("settings.json"))
}

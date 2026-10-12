#[derive(Debug, Clone)]
pub struct SettingsState {
    pub hotkey_mode: i32,
    pub hotkey_combo_ctrl_required: bool,
    pub hotkey_combo_shift_required: bool,
    pub hotkey_combo_key: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            hotkey_mode: 0,
            hotkey_combo_ctrl_required: true,
            hotkey_combo_shift_required: false,
            hotkey_combo_key: "H".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HotkeySettings {
    pub hotkey_mode: i32,
    pub combo_ctrl_required: bool,
    pub combo_shift_required: bool,
    pub combo_key: String,
}

impl SettingsState {
    pub fn hotkey_settings(&self) -> HotkeySettings {
        HotkeySettings {
            hotkey_mode: self.hotkey_mode,
            combo_ctrl_required: self.hotkey_combo_ctrl_required,
            combo_shift_required: self.hotkey_combo_shift_required,
            combo_key: self.hotkey_combo_key.clone(),
        }
    }

    pub fn set_hotkey_mode(&mut self, mode: i32) {
        self.hotkey_mode = mode.clamp(0, 2);
    }

    pub fn set_combo_key(&mut self, input: &str) {
        self.hotkey_combo_key = normalize_combo_key(input);
    }
}

pub fn normalize_combo_key(input: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_normalization_preserves_existing_rules() {
        let mut state = SettingsState::default();
        for (input, expected) in [
            (" j ", "J"),
            ("7", "7"),
            ("abc", "A"),
            ("", "H"),
            ("あ", "H"),
        ] {
            state.set_combo_key(input);
            assert_eq!(state.hotkey_settings().combo_key, expected);
        }
    }

    #[test]
    fn hotkey_mode_is_clamped_to_supported_range() {
        let mut state = SettingsState::default();
        for (input, expected) in [(-1, 0), (1, 1), (2, 2), (3, 2)] {
            state.set_hotkey_mode(input);
            assert_eq!(state.hotkey_settings().hotkey_mode, expected);
        }
    }
}

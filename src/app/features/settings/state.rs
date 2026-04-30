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
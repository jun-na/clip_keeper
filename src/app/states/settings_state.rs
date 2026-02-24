// 永続化対象となる設定状態。
#[derive(Debug, Clone)]
pub struct SettingsState {
    pub hotkey_ctrl_double_tap_enabled: bool,
    pub hotkey_shift_double_tap_enabled: bool,
    pub hotkey_combo_ctrl_required: bool,
    pub hotkey_combo_shift_required: bool,
    pub hotkey_combo_key: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            hotkey_ctrl_double_tap_enabled: true,
            hotkey_shift_double_tap_enabled: true,
            hotkey_combo_ctrl_required: true,
            hotkey_combo_shift_required: true,
            hotkey_combo_key: "H".to_string(),
        }
    }
}

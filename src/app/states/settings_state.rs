// 永続化対象となる設定状態。
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// ホットキーモード: 0=Shift 2回押し, 1=Ctrl 2回押し, 2=修飾キー+ホットキー
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

use self::{HotkeyKey as Key, KeyEvent as EventType};
use super::settings::HotkeySettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyKey {
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    KeyPress(HotkeyKey),
    KeyRelease(HotkeyKey),
}

use std::time::{Duration, Instant};

const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(450);

#[derive(Default)]
pub struct DoubleTapDetector {
    last_tap: Option<Instant>,
}

impl DoubleTapDetector {
    pub fn register_tap(&mut self, now: Instant) -> bool {
        let fired = self
            .last_tap
            .map(|last| now.duration_since(last) <= DOUBLE_TAP_WINDOW)
            .unwrap_or(false);
        self.last_tap = Some(now);
        fired
    }
}

#[derive(Default)]
pub struct HotkeyState {
    shift_double_tap: DoubleTapDetector,
    ctrl_double_tap: DoubleTapDetector,
    ctrl_down: bool,
    shift_down: bool,
    combo_key_down: bool,
}

impl HotkeyState {
    pub fn handle_event(
        &mut self,
        event_type: KeyEvent,
        settings: &HotkeySettings,
        at: Instant,
    ) -> Option<String> {
        match event_type {
            EventType::KeyPress(key) => match key {
                Key::ShiftLeft | Key::ShiftRight => {
                    if !self.shift_down {
                        self.shift_down = true;
                    }
                }
                Key::ControlLeft | Key::ControlRight => {
                    if !self.ctrl_down {
                        self.ctrl_down = true;
                    }
                }
                _ => {
                    if settings.hotkey_mode == 2 && is_combo_key(key, &settings.combo_key) {
                        if !self.combo_key_down {
                            self.combo_key_down = true;
                            let ctrl_ok = !settings.combo_ctrl_required || self.ctrl_down;
                            let shift_ok = !settings.combo_shift_required || self.shift_down;
                            if ctrl_ok && shift_ok {
                                return Some(format!(
                                    "Combo key ({key:?}) ctrl:{} shift:{}",
                                    self.ctrl_down, self.shift_down
                                ));
                            }
                        }
                    }
                }
            },
            EventType::KeyRelease(key) => match key {
                Key::ShiftLeft | Key::ShiftRight => {
                    self.shift_down = false;

                    if settings.hotkey_mode == 0 && self.shift_double_tap.register_tap(at) {
                        return Some(format!("Shift double-tap ({key:?})"));
                    }
                }
                Key::ControlLeft | Key::ControlRight => {
                    self.ctrl_down = false;

                    if settings.hotkey_mode == 1 && self.ctrl_double_tap.register_tap(at) {
                        return Some(format!("Ctrl double-tap ({key:?})"));
                    }
                }
                _ => {
                    if is_combo_key(key, &settings.combo_key) {
                        self.combo_key_down = false;
                    }
                }
            },
        }
        None
    }
}

fn is_combo_key(key: Key, configured_key: &str) -> bool {
    if configured_key.is_empty() {
        return key == Key::KeyH;
    }

    let c = configured_key
        .chars()
        .next()
        .unwrap_or('H')
        .to_ascii_uppercase();
    match c {
        'A' => key == Key::KeyA,
        'B' => key == Key::KeyB,
        'C' => key == Key::KeyC,
        'D' => key == Key::KeyD,
        'E' => key == Key::KeyE,
        'F' => key == Key::KeyF,
        'G' => key == Key::KeyG,
        'H' => key == Key::KeyH,
        'I' => key == Key::KeyI,
        'J' => key == Key::KeyJ,
        'K' => key == Key::KeyK,
        'L' => key == Key::KeyL,
        'M' => key == Key::KeyM,
        'N' => key == Key::KeyN,
        'O' => key == Key::KeyO,
        'P' => key == Key::KeyP,
        'Q' => key == Key::KeyQ,
        'R' => key == Key::KeyR,
        'S' => key == Key::KeyS,
        'T' => key == Key::KeyT,
        'U' => key == Key::KeyU,
        'V' => key == Key::KeyV,
        'W' => key == Key::KeyW,
        'X' => key == Key::KeyX,
        'Y' => key == Key::KeyY,
        'Z' => key == Key::KeyZ,
        '0' => key == Key::Num0,
        '1' => key == Key::Num1,
        '2' => key == Key::Num2,
        '3' => key == Key::Num3,
        '4' => key == Key::Num4,
        '5' => key == Key::Num5,
        '6' => key == Key::Num6,
        '7' => key == Key::Num7,
        '8' => key == Key::Num8,
        '9' => key == Key::Num9,
        _ => key == Key::KeyH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::domain::settings::HotkeySettings;

    fn settings(mode: i32) -> HotkeySettings {
        HotkeySettings {
            hotkey_mode: mode,
            combo_ctrl_required: true,
            combo_shift_required: false,
            combo_key: "H".into(),
        }
    }

    #[test]
    fn double_tap_uses_event_timestamps() {
        let mut state = HotkeyState::default();
        let settings = settings(0);
        let at = Instant::now();
        let event = EventType::KeyRelease(Key::ShiftLeft);
        assert!(state.handle_event(event, &settings, at).is_none());
        // Dispatch both old events later: their original spacing still decides the result.
        assert!(state
            .handle_event(event, &settings, at + Duration::from_millis(451))
            .is_none());
        assert!(state
            .handle_event(event, &settings, at + Duration::from_millis(901))
            .is_some());
    }

    #[test]
    fn double_tap_respects_selected_modifier() {
        let mut state = HotkeyState::default();
        let settings = settings(1);
        let at = Instant::now();
        for offset in [0, 100] {
            assert!(state
                .handle_event(
                    EventType::KeyRelease(Key::ShiftLeft),
                    &settings,
                    at + Duration::from_millis(offset)
                )
                .is_none());
        }
        assert!(state
            .handle_event(
                EventType::KeyRelease(Key::ControlRight),
                &settings,
                at + Duration::from_millis(200)
            )
            .is_none());
        assert!(state
            .handle_event(
                EventType::KeyRelease(Key::ControlRight),
                &settings,
                at + Duration::from_millis(300)
            )
            .is_some());
    }

    #[test]
    fn combo_requires_modifiers_and_suppresses_repeat() {
        let mut state = HotkeyState::default();
        let mut settings = settings(2);
        settings.combo_shift_required = true;
        settings.combo_key = "j".into();
        let at = Instant::now();
        assert!(state
            .handle_event(EventType::KeyPress(Key::KeyJ), &settings, at)
            .is_none());
        state.handle_event(EventType::KeyRelease(Key::KeyJ), &settings, at);
        state.handle_event(EventType::KeyPress(Key::ControlLeft), &settings, at);
        assert!(state
            .handle_event(EventType::KeyPress(Key::KeyJ), &settings, at)
            .is_none());
        state.handle_event(EventType::KeyRelease(Key::KeyJ), &settings, at);
        state.handle_event(EventType::KeyPress(Key::ShiftRight), &settings, at);
        assert!(state
            .handle_event(EventType::KeyPress(Key::KeyJ), &settings, at)
            .is_some());
        assert!(state
            .handle_event(EventType::KeyPress(Key::KeyJ), &settings, at)
            .is_none());
        state.handle_event(EventType::KeyRelease(Key::KeyJ), &settings, at);
        assert!(state
            .handle_event(EventType::KeyPress(Key::KeyJ), &settings, at)
            .is_some());
    }

    #[test]
    fn setting_change_applies_to_next_event() {
        let mut state = HotkeyState::default();
        let mut settings = settings(0);
        let at = Instant::now();
        let event = EventType::KeyRelease(Key::ShiftLeft);
        assert!(state.handle_event(event, &settings, at).is_none());
        settings.hotkey_mode = 1;
        assert!(state
            .handle_event(event, &settings, at + Duration::from_millis(100))
            .is_none());
    }
}

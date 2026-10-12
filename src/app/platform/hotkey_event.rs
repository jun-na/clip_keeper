use crate::app::domain::hotkey::{HotkeyKey, KeyEvent};
use rdev::{EventType, Key};

pub fn to_domain_event(event: EventType) -> Option<KeyEvent> {
    match event {
        EventType::KeyPress(key) => Some(KeyEvent::KeyPress(to_domain_key(key))),
        EventType::KeyRelease(key) => Some(KeyEvent::KeyRelease(to_domain_key(key))),
        _ => None,
    }
}

fn to_domain_key(key: Key) -> HotkeyKey {
    match key {
        Key::ShiftLeft => HotkeyKey::ShiftLeft,
        Key::ShiftRight => HotkeyKey::ShiftRight,
        Key::ControlLeft => HotkeyKey::ControlLeft,
        Key::ControlRight => HotkeyKey::ControlRight,
        Key::KeyA => HotkeyKey::KeyA,
        Key::KeyB => HotkeyKey::KeyB,
        Key::KeyC => HotkeyKey::KeyC,
        Key::KeyD => HotkeyKey::KeyD,
        Key::KeyE => HotkeyKey::KeyE,
        Key::KeyF => HotkeyKey::KeyF,
        Key::KeyG => HotkeyKey::KeyG,
        Key::KeyH => HotkeyKey::KeyH,
        Key::KeyI => HotkeyKey::KeyI,
        Key::KeyJ => HotkeyKey::KeyJ,
        Key::KeyK => HotkeyKey::KeyK,
        Key::KeyL => HotkeyKey::KeyL,
        Key::KeyM => HotkeyKey::KeyM,
        Key::KeyN => HotkeyKey::KeyN,
        Key::KeyO => HotkeyKey::KeyO,
        Key::KeyP => HotkeyKey::KeyP,
        Key::KeyQ => HotkeyKey::KeyQ,
        Key::KeyR => HotkeyKey::KeyR,
        Key::KeyS => HotkeyKey::KeyS,
        Key::KeyT => HotkeyKey::KeyT,
        Key::KeyU => HotkeyKey::KeyU,
        Key::KeyV => HotkeyKey::KeyV,
        Key::KeyW => HotkeyKey::KeyW,
        Key::KeyX => HotkeyKey::KeyX,
        Key::KeyY => HotkeyKey::KeyY,
        Key::KeyZ => HotkeyKey::KeyZ,
        Key::Num0 => HotkeyKey::Num0,
        Key::Num1 => HotkeyKey::Num1,
        Key::Num2 => HotkeyKey::Num2,
        Key::Num3 => HotkeyKey::Num3,
        Key::Num4 => HotkeyKey::Num4,
        Key::Num5 => HotkeyKey::Num5,
        Key::Num6 => HotkeyKey::Num6,
        Key::Num7 => HotkeyKey::Num7,
        Key::Num8 => HotkeyKey::Num8,
        Key::Num9 => HotkeyKey::Num9,
        _ => HotkeyKey::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_keyboard_events_at_platform_boundary() {
        for (platform, domain) in [
            (Key::ShiftRight, HotkeyKey::ShiftRight),
            (Key::ControlLeft, HotkeyKey::ControlLeft),
            (Key::KeyJ, HotkeyKey::KeyJ),
            (Key::Num7, HotkeyKey::Num7),
        ] {
            assert!(matches!(to_domain_event(EventType::KeyPress(platform)),
                Some(KeyEvent::KeyPress(key)) if key == domain));
            assert!(matches!(to_domain_event(EventType::KeyRelease(platform)),
                Some(KeyEvent::KeyRelease(key)) if key == domain));
        }
    }

    #[test]
    fn excludes_mouse_events_from_hotkey_requests() {
        assert!(to_domain_event(EventType::MouseMove { x: 10.0, y: 20.0 }).is_none());
    }
}

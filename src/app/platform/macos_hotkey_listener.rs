use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
};
use objc2::rc::autoreleasepool;
use rdev::{EventType, Key};

#[derive(Debug)]
pub enum MacHotkeyListenError {
    EventTapError,
    LoopSourceError,
}

pub fn listen<T>(callback: T) -> Result<(), MacHotkeyListenError>
where
    T: FnMut(EventType) + Send + 'static,
{
    autoreleasepool(|_| {
        let callback = Mutex::new(callback);
        let last_flags = AtomicU64::new(CGEventFlags::CGEventFlagNull.bits());
        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
            ],
            move |_proxy, event_type, event| {
                if let Some(converted) = convert_event(event_type, event, &last_flags) {
                    callback.lock().expect("hotkey callback lock poisoned")(converted);
                }
                CallbackResult::Keep
            },
        )
        .map_err(|_| MacHotkeyListenError::EventTapError)?;

        let source = tap
            .mach_port()
            .create_runloop_source(0)
            .map_err(|_| MacHotkeyListenError::LoopSourceError)?;
        let run_loop = CFRunLoop::get_current();
        // Core Foundationが提供する共通モード定数を読み取る。
        run_loop.add_source(&source, unsafe { kCFRunLoopCommonModes });
        tap.enable();
        CFRunLoop::run_current();
        Ok(())
    })
}

fn convert_event(
    event_type: CGEventType,
    cg_event: &CGEvent,
    last_flags: &AtomicU64,
) -> Option<EventType> {
    let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
    let key = key_from_code(code);

    match event_type {
        CGEventType::KeyDown => Some(EventType::KeyPress(key)),
        CGEventType::KeyUp => Some(EventType::KeyRelease(key)),
        CGEventType::FlagsChanged => {
            let current_flags = cg_event.get_flags();
            let current_bits = current_flags.bits();
            let previous_bits = last_flags.swap(current_bits, Ordering::Relaxed);

            if current_bits < previous_bits {
                Some(EventType::KeyRelease(key))
            } else {
                Some(EventType::KeyPress(key))
            }
        }
        _ => None,
    }
}

fn key_from_code(code: u16) -> Key {
    match code {
        0 => Key::KeyA,
        1 => Key::KeyS,
        2 => Key::KeyD,
        3 => Key::KeyF,
        4 => Key::KeyH,
        5 => Key::KeyG,
        6 => Key::KeyZ,
        7 => Key::KeyX,
        8 => Key::KeyC,
        9 => Key::KeyV,
        11 => Key::KeyB,
        12 => Key::KeyQ,
        13 => Key::KeyW,
        14 => Key::KeyE,
        15 => Key::KeyR,
        16 => Key::KeyY,
        17 => Key::KeyT,
        18 => Key::Num1,
        19 => Key::Num2,
        20 => Key::Num3,
        21 => Key::Num4,
        22 => Key::Num6,
        23 => Key::Num5,
        24 => Key::Equal,
        25 => Key::Num9,
        26 => Key::Num7,
        27 => Key::Minus,
        28 => Key::Num8,
        29 => Key::Num0,
        30 => Key::RightBracket,
        31 => Key::KeyO,
        32 => Key::KeyU,
        33 => Key::LeftBracket,
        34 => Key::KeyI,
        35 => Key::KeyP,
        37 => Key::KeyL,
        38 => Key::KeyJ,
        39 => Key::Quote,
        40 => Key::KeyK,
        41 => Key::SemiColon,
        42 => Key::BackSlash,
        43 => Key::Comma,
        44 => Key::Slash,
        45 => Key::KeyN,
        46 => Key::KeyM,
        47 => Key::Dot,
        56 => Key::ShiftLeft,
        59 => Key::ControlLeft,
        60 => Key::ShiftRight,
        62 => Key::ControlRight,
        code => Key::Unknown(code.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    fn keyboard_event(code: u16) -> CGEvent {
        let source = CGEventSource::new(CGEventSourceStateID::Private).unwrap();
        CGEvent::new_keyboard_event(source, code, true).unwrap()
    }

    #[test]
    fn converts_borrowed_keyboard_events_without_consuming_them() {
        let event = keyboard_event(0);
        let flags = AtomicU64::new(0);
        assert!(matches!(
            convert_event(CGEventType::KeyDown, &event, &flags),
            Some(EventType::KeyPress(Key::KeyA))
        ));
        assert!(matches!(
            convert_event(CGEventType::KeyUp, &event, &flags),
            Some(EventType::KeyRelease(Key::KeyA))
        ));
        assert!(convert_event(CGEventType::MouseMoved, &event, &flags).is_none());
        assert_eq!(
            event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE),
            0
        );
    }

    #[test]
    fn modifier_state_is_local_to_each_listener() {
        let event = keyboard_event(59);
        let first_flags = AtomicU64::new(0);
        let second_flags = AtomicU64::new(0);
        event.set_flags(CGEventFlags::CGEventFlagControl);
        assert!(matches!(
            convert_event(CGEventType::FlagsChanged, &event, &first_flags),
            Some(EventType::KeyPress(Key::ControlLeft))
        ));
        assert_eq!(second_flags.load(Ordering::Relaxed), 0);
        assert!(matches!(
            convert_event(CGEventType::FlagsChanged, &event, &second_flags),
            Some(EventType::KeyPress(Key::ControlLeft))
        ));
        event.set_flags(CGEventFlags::CGEventFlagNull);
        assert!(matches!(
            convert_event(CGEventType::FlagsChanged, &event, &first_flags),
            Some(EventType::KeyRelease(Key::ControlLeft))
        ));
    }
}

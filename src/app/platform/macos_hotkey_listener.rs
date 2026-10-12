use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use cocoa::base::{id, nil};
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, EventField};
use rdev::EventType;
use rdev::Key;

type CFMachPortRef = *const c_void;
type CFRunLoopMode = id;
type CFRunLoopRef = id;
type CFRunLoopSourceRef = id;
type CGEventRef = CGEvent;
type CGEventTapPlacement = u32;
type CGEventTapProxy = id;
type CGEventMask = u64;

const KCG_HEAD_INSERT_EVENT_TAP: CGEventTapPlacement = 0;
const KEY_EVENT_MASK: CGEventMask = (1 << CGEventType::KeyDown as u64)
    | (1 << CGEventType::KeyUp as u64)
    | (1 << CGEventType::FlagsChanged as u64);

static GLOBAL_CALLBACK: Mutex<Option<Box<dyn FnMut(EventType) + Send>>> = Mutex::new(None);
static LAST_FLAGS: AtomicU64 = AtomicU64::new(CGEventFlags::CGEventFlagNull.bits());

#[derive(Debug)]
pub enum MacHotkeyListenError {
    EventTapError,
    LoopSourceError,
}

#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOption,
        events_of_interest: CGEventMask,
        callback: EventTapCallback,
        user_info: id,
    ) -> CFMachPortRef;
    fn CFMachPortCreateRunLoopSource(
        allocator: id,
        tap: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CFRunLoopRun();

    static kCFRunLoopCommonModes: CFRunLoopMode;
}

#[repr(u32)]
enum CGEventTapOption {
    ListenOnly = 1,
}

type EventTapCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if let Some(converted) = convert_event(event_type, &cg_event) {
        let mut global_callback = GLOBAL_CALLBACK
            .lock()
            .expect("hotkey callback lock poisoned");
        if let Some(callback) = global_callback.as_mut() {
            callback(converted);
        }
    }
    cg_event
}

pub fn listen<T>(callback: T) -> Result<(), MacHotkeyListenError>
where
    T: FnMut(EventType) + Send + 'static,
{
    unsafe {
        let mut global_callback = GLOBAL_CALLBACK
            .lock()
            .expect("hotkey callback lock poisoned");
        *global_callback = Some(Box::new(callback));
        drop(global_callback);
        LAST_FLAGS.store(CGEventFlags::CGEventFlagNull.bits(), Ordering::Relaxed);

        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID,
            KCG_HEAD_INSERT_EVENT_TAP,
            CGEventTapOption::ListenOnly,
            KEY_EVENT_MASK,
            raw_callback,
            nil,
        );
        if tap.is_null() {
            return Err(MacHotkeyListenError::EventTapError);
        }

        let run_loop_source = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if run_loop_source.is_null() {
            return Err(MacHotkeyListenError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, run_loop_source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }

    Ok(())
}

unsafe fn convert_event(event_type: CGEventType, cg_event: &CGEvent) -> Option<EventType> {
    let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
    let key = key_from_code(code);

    match event_type {
        CGEventType::KeyDown => Some(EventType::KeyPress(key)),
        CGEventType::KeyUp => Some(EventType::KeyRelease(key)),
        CGEventType::FlagsChanged => {
            let current_flags = cg_event.get_flags();
            let current_bits = current_flags.bits();
            let previous_bits = LAST_FLAGS.swap(current_bits, Ordering::Relaxed);

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

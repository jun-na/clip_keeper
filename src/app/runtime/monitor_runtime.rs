use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use rdev::{EventType, Key};

use crate::app::features::clipboard::service::ClipboardService;
use crate::app::features::settings::service::SettingsService;
#[cfg(target_os = "macos")]
use crate::app::platform::macos_hotkey_listener;
use crate::app::runtime::detectors::DoubleTapDetector;
use crate::app::runtime::hotkey_logger::HotkeyLogger;
use crate::app::runtime::ui_gateway::UiGateway;

const POLL_INTERVAL: Duration = Duration::from_millis(120);

pub struct MonitorRuntime {
    clipboard_service: Arc<ClipboardService>,
    settings_service: Arc<SettingsService>,
    ui_gateway: Arc<UiGateway>,
}

impl MonitorRuntime {
    pub fn new(
        clipboard_service: Arc<ClipboardService>,
        settings_service: Arc<SettingsService>,
        ui_gateway: Arc<UiGateway>,
    ) -> Self {
        Self {
            clipboard_service,
            settings_service,
            ui_gateway,
        }
    }

    pub fn start(&self) {
        self.start_clipboard_thread();
        self.start_hotkey_thread();
    }

    fn start_clipboard_thread(&self) {
        let clipboard_service = self.clipboard_service.clone();
        let ui_gateway = self.ui_gateway.clone();

        thread::spawn(move || {
            let mut clipboard = Clipboard::new().ok();

            loop {
                if let Some(cb) = clipboard.as_mut() {
                    if let Ok(text) = cb.get_text() {
                        let changed = clipboard_service.push_clipboard_text(text);
                        if changed {
                            ui_gateway.refresh_history_model();
                        }
                    }
                } else {
                    clipboard = Clipboard::new().ok();
                }

                thread::sleep(POLL_INTERVAL);
            }
        });
    }

    fn start_hotkey_thread(&self) {
        let settings_service = self.settings_service.clone();
        let ui_gateway = self.ui_gateway.clone();

        thread::spawn(move || {
            let logger = HotkeyLogger::new();
            let mut shift_double_tap = DoubleTapDetector::default();
            let mut ctrl_double_tap = DoubleTapDetector::default();

            let mut ctrl_down = false;
            let mut shift_down = false;
            let mut combo_key_down = false;

            let mut handle_event = move |event_type: EventType| match event_type {
                EventType::KeyPress(key) => {
                    let settings = settings_service.current_hotkey_settings();

                    match key {
                        Key::ShiftLeft | Key::ShiftRight => {
                            if !shift_down {
                                shift_down = true;
                            }
                        }
                        Key::ControlLeft | Key::ControlRight => {
                            if !ctrl_down {
                                ctrl_down = true;
                            }
                        }
                        _ => {
                            if settings.hotkey_mode == 2 && is_combo_key(key, &settings.combo_key) {
                                if !combo_key_down {
                                    combo_key_down = true;
                                    let ctrl_ok = !settings.combo_ctrl_required || ctrl_down;
                                    let shift_ok = !settings.combo_shift_required || shift_down;
                                    if ctrl_ok && shift_ok {
                                        logger.log(&format!(
                                            "Combo key ({key:?}) ctrl:{ctrl_down} shift:{shift_down}"
                                        ));
                                        ui_gateway.show_history_window();
                                    }
                                }
                            }
                        }
                    }
                }
                EventType::KeyRelease(key) => match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        shift_down = false;
                        let settings = settings_service.current_hotkey_settings();
                        if settings.hotkey_mode == 0
                            && shift_double_tap.register_tap(Instant::now())
                        {
                            logger.log(&format!("Shift double-tap ({key:?})"));
                            ui_gateway.show_history_window();
                        }
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        ctrl_down = false;
                        let settings = settings_service.current_hotkey_settings();
                        if settings.hotkey_mode == 1 && ctrl_double_tap.register_tap(Instant::now())
                        {
                            logger.log(&format!("Ctrl double-tap ({key:?})"));
                            ui_gateway.show_history_window();
                        }
                    }
                    _ => {
                        let settings = settings_service.current_hotkey_settings();
                        if is_combo_key(key, &settings.combo_key) {
                            combo_key_down = false;
                        }
                    }
                },
                _ => {}
            };

            #[cfg(target_os = "macos")]
            if let Err(error) = macos_hotkey_listener::listen(handle_event) {
                eprintln!("global hotkey listener failed: {error:?}");
            }

            #[cfg(not(target_os = "macos"))]
            if let Err(error) = rdev::listen(move |event| handle_event(event.event_type)) {
                eprintln!("global hotkey listener failed: {error:?}");
            }
        });
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
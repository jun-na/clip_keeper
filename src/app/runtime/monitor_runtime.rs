use crate::app::platform::hotkey_event::to_domain_event;
#[cfg(target_os = "macos")]
use crate::app::platform::macos_hotkey_listener;
use crate::app::request::{Request, RequestSender};
use arboard::Clipboard;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(120);

/// 起動した監視スレッドのハンドル。OSリスナーはプロセス終了まで動く。
pub struct MonitorRuntime {
    _clipboard_thread: thread::JoinHandle<()>,
    _hotkey_thread: thread::JoinHandle<()>,
}

impl MonitorRuntime {
    pub fn start(send_request: RequestSender) -> Self {
        Self {
            _clipboard_thread: start_clipboard_thread(send_request.clone()),
            _hotkey_thread: start_hotkey_thread(send_request),
        }
    }
}

fn start_clipboard_thread(send_request: RequestSender) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut clipboard = Clipboard::new().ok();
        loop {
            if let Some(cb) = clipboard.as_mut() {
                if let Ok(text) = cb.get_text() {
                    send_request(Request::ClipboardObserved { text });
                }
            } else {
                clipboard = Clipboard::new().ok();
            }
            thread::sleep(POLL_INTERVAL);
        }
    })
}

fn start_hotkey_thread(send_request: RequestSender) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let handle_event = move |event| {
            if let Some(event) = to_domain_event(event) {
                send_request(Request::HotkeyEvent {
                    event,
                    at: Instant::now(),
                });
            }
        };
        #[cfg(target_os = "macos")]
        if let Err(error) = macos_hotkey_listener::listen(handle_event) {
            eprintln!("global hotkey listener failed: {error:?}");
        }
        #[cfg(not(target_os = "macos"))]
        if let Err(error) = rdev::listen(move |event| handle_event(event.event_type)) {
            eprintln!("global hotkey listener failed: {error:?}");
        }
    })
}

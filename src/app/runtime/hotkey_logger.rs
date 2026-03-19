use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HotkeyLogger {
    log_path: PathBuf,
    enabled: AtomicBool,
}

impl HotkeyLogger {
    pub fn new() -> Self {
        Self::new_with_enabled(false)
    }

    pub fn new_with_enabled(enabled: bool) -> Self {
        let log_path = default_log_path().unwrap_or_else(|_| PathBuf::from("hotkey_log.txt"));
        Self {
            log_path,
            enabled: AtomicBool::new(enabled),
        }
    }

    #[allow(dead_code)]
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn enable(&self) {
        self.set_enabled(true);
    }

    #[allow(dead_code)]
    pub fn disable(&self) {
        self.set_enabled(false);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn log(&self, trigger: &str) {
        if !self.is_enabled() {
            return;
        }

        let timestamp = format_utc_now();
        let line = format!("[{timestamp}] Trigger: {trigger}\n");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

fn format_utc_now() -> String {
    let total_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let s = total_secs % 60;
    let m = (total_secs / 60) % 60;
    let h = (total_secs / 3600) % 24;
    let days = total_secs / 86400;

    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02} {h:02}:{m:02}:{s:02} UTC")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn default_log_path() -> io::Result<PathBuf> {
    crate::app::storage_paths::data_file_path("hotkey_log.txt")
}
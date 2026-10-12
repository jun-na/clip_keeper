use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{AccessLogPath, AccessState};

#[entrait::entrait(pub LoggingService)]
mod operations {
    use super::*;

    #[allow(dead_code)]
    pub fn set_hotkey_logging_enabled(deps: &impl AccessState, enabled: bool) {
        *deps
            .state()
            .hotkey_log_enabled
            .lock()
            .expect("logging state lock poisoned") = enabled;
    }

    pub fn log_hotkey(deps: &(impl AccessState + AccessLogPath), trigger: &str) {
        let enabled = *deps
            .state()
            .hotkey_log_enabled
            .lock()
            .expect("logging state lock poisoned");
        if !enabled {
            return;
        }

        let timestamp = format_utc_at(SystemTime::now());
        let line = format!("[{timestamp}] Trigger: {trigger}\n");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(deps.hotkey_log_path())
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

fn format_utc_at(at: SystemTime) -> String {
    let total_secs = at
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use std::path::{Path, PathBuf};

    struct LogDependencies {
        state: AppState,
        path: PathBuf,
    }

    impl AccessState for LogDependencies {
        fn state(&self) -> &AppState {
            &self.state
        }
    }

    impl AccessLogPath for LogDependencies {
        fn hotkey_log_path(&self) -> &Path {
            &self.path
        }
    }

    #[test]
    fn logging_respects_shared_enabled_state_and_appends_to_injected_path() {
        let path = std::env::temp_dir().join(format!(
            "clip-keeper-log-{}-{}.txt",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let deps = entrait::Impl::new(LogDependencies {
            state: AppState::new(),
            path: path.clone(),
        });
        deps.log_hotkey("disabled");
        assert!(!path.exists());

        deps.set_hotkey_logging_enabled(true);
        deps.log_hotkey("first");
        deps.log_hotkey("second");
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with('['));
        assert!(lines[0].ends_with(" UTC] Trigger: first"));
        assert!(lines[1].ends_with(" UTC] Trigger: second"));

        deps.set_hotkey_logging_enabled(false);
        deps.log_hotkey("disabled again");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
        std::fs::remove_file(path).unwrap();
    }
}

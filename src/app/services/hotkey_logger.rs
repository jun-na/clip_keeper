use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// ホットキートリガーのログをファイルへ追記するロガー。
/// ログファイルは実行ファイルと同じディレクトリに `hotkey_log.txt` として保存される。
pub struct HotkeyLogger {
    log_path: PathBuf,
}

impl HotkeyLogger {
    /// 実行ファイルのディレクトリを基準にログパスを決定して生成する。
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            log_path: exe_dir.join("hotkey_log.txt"),
        }
    }

    /// トリガー名を含む1行のログエントリを追記する。
    /// ファイルが存在しない場合は新規作成する。書き込み失敗は静かに無視する。
    pub fn log(&self, trigger: &str) {
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

/// 現在の UTC 時刻を `YYYY-MM-DD HH:MM:SS UTC` 形式で返す。
fn format_utc_now() -> String {
    let total_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let s = total_secs % 60;
    let m = (total_secs / 60) % 60;
    let h = (total_secs / 3600) % 24;
    let days = total_secs / 86400;

    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02} {h:02}:{m:02}:{s:02} UTC")
}

/// Unix エポック起算の日数をグレゴリオ暦の (年, 月, 日) に変換する。
/// 参考: Euclidean Affine Functions (Cassini algorithm)
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

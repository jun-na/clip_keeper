use std::time::{Duration, Instant};

const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(450);

// ダブルタップ（短時間で2回押し）を判定する小さな状態機械。
#[derive(Default)]
pub struct DoubleTapDetector {
    last_tap: Option<Instant>,
}

impl DoubleTapDetector {
    /// 1回押下を登録し、前回押下との間隔が閾値内なら true を返す。
    pub fn register_tap(&mut self, now: Instant) -> bool {
        let fired = self
            .last_tap
            .map(|last| now.duration_since(last) <= DOUBLE_TAP_WINDOW)
            .unwrap_or(false);
        self.last_tap = Some(now);
        fired
    }
}

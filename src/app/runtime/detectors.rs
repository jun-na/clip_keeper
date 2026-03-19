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
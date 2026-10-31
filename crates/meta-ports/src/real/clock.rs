use crate::Clock;
use meta_core::{Span, Timestamp};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_secs()).unwrap_or_default();
        Timestamp::from_unix_seconds(seconds)
    }

    fn pause(&self, span: Span) {
        std::thread::sleep(std::time::Duration::from_secs(span.whole_seconds()));
    }
}

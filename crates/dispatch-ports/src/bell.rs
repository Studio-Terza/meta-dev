use std::sync::{Arc, Condvar, Mutex, WaitTimeoutResult};
use std::time::Duration;

use invariants::Unpoisoned;

#[derive(Debug)]
pub struct Bell(Arc<Peal>);

#[derive(Debug)]
pub(crate) struct Ringer(Arc<Peal>);

#[derive(Debug, Default)]
struct Peal {
    chime: Mutex<Chime>,
    heard: Condvar,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Chime {
    #[default]
    Quiet,
    Rung,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Woke {
    Lapsed,
    Rung,
}

impl Bell {
    #[must_use]
    pub fn hung() -> Bell {
        Bell(Arc::default())
    }

    #[must_use]
    pub(crate) fn ringer(&self) -> Ringer {
        Ringer(Arc::clone(&self.0))
    }

    #[must_use]
    pub fn waited(&self, pause: Duration) -> Woke {
        let listening = self.0.chime.lock().unpoisoned();
        let (mut heard, lapsed) =
            self.0.heard.wait_timeout_while(listening, pause, |chime| *chime == Chime::Quiet).unpoisoned();
        *heard = Chime::Quiet;
        Woke::from(lapsed)
    }
}

impl Ringer {
    pub(crate) fn ring(&self) {
        *self.0.chime.lock().unpoisoned() = Chime::Rung;
        self.0.heard.notify_all();
    }
}

impl From<WaitTimeoutResult> for Woke {
    fn from(lapsed: WaitTimeoutResult) -> Woke {
        Some(lapsed).filter(WaitTimeoutResult::timed_out).map_or(Woke::Rung, |_lapsed| Woke::Lapsed)
    }
}

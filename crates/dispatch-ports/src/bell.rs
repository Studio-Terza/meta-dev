use std::sync::{Arc, Condvar, Mutex, PoisonError, WaitTimeoutResult};
use std::time::Duration;

#[derive(Debug)]
pub struct Bell(Arc<Peal>);

#[derive(Debug)]
pub struct Ringer(Arc<Peal>);

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
    pub(crate) fn hung() -> Bell {
        Bell(Arc::default())
    }

    #[must_use]
    pub fn ringer(&self) -> Ringer {
        Ringer(Arc::clone(&self.0))
    }

    pub fn waited(&self, pause: Duration) -> Woke {
        let chime = self.0.chime.lock().unwrap_or_else(PoisonError::into_inner);
        let (mut chime, lapsed) = self
            .0
            .heard
            .wait_timeout_while(chime, pause, |chime| *chime == Chime::Quiet)
            .unwrap_or_else(PoisonError::into_inner);
        *chime = Chime::Quiet;
        Woke::from(lapsed)
    }
}

impl Ringer {
    pub fn ring(&self) {
        *self.0.chime.lock().unwrap_or_else(PoisonError::into_inner) = Chime::Rung;
        self.0.heard.notify_all();
    }
}

impl From<WaitTimeoutResult> for Woke {
    fn from(lapsed: WaitTimeoutResult) -> Woke {
        Some(lapsed).filter(WaitTimeoutResult::timed_out).map_or(Woke::Rung, |_lapsed| Woke::Lapsed)
    }
}

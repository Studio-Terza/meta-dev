use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

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

impl Bell {
    pub(crate) fn hung() -> Bell {
        Bell(Arc::default())
    }

    pub(crate) fn ringer(&self) -> Ringer {
        Ringer(Arc::clone(&self.0))
    }

    pub(crate) fn waited(&self, pause: Duration) {
        let chime = self.0.chime.lock().unwrap_or_else(PoisonError::into_inner);
        let (mut chime, _lapsed) = self
            .0
            .heard
            .wait_timeout_while(chime, pause, |chime| *chime == Chime::Quiet)
            .unwrap_or_else(PoisonError::into_inner);
        *chime = Chime::Quiet;
    }
}

impl Ringer {
    pub(crate) fn ring(&self) {
        *self.0.chime.lock().unwrap_or_else(PoisonError::into_inner) = Chime::Rung;
        self.0.heard.notify_all();
    }
}

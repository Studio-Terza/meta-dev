use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct Bell(Arc<Peal>);

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
    pub(crate) fn ring(&self) {
        *self.0.chime.lock().unwrap_or_else(PoisonError::into_inner) = Chime::Rung;
        self.0.heard.notify_all();
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

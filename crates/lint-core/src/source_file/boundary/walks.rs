use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct Walks(AtomicUsize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Repeated(usize);

impl Walks {
    pub fn walked(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub fn repeated(&self) -> Option<Repeated> {
        Some(self.0.load(Ordering::Relaxed)).filter(|count| *count > 1).map(Repeated)
    }
}

impl fmt::Display for Repeated {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

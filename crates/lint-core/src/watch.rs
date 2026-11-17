mod boundary;
mod facts;

use crate::walk::Walk;

pub use boundary::Address;
pub use facts::Facts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Descend {
    Into,
    Over,
    Expand,
    Unfold,
}

impl From<Walk> for Descend {
    fn from(walk: Walk) -> Descend {
        match walk {
            Walk::Entered => Descend::Into,
            Walk::Skipped => Descend::Over,
        }
    }
}

impl Descend {
    #[must_use]
    pub fn entered(self) -> Option<Descend> {
        Some(self).filter(|descend| *descend == Descend::Into)
    }

    #[must_use]
    pub fn then(self, next: impl FnOnce() -> Descend) -> Descend {
        self.entered().map_or(self, |_| next())
    }
}

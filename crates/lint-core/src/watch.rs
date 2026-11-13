mod boundary;
mod hooks;

use crate::walk::Walk;

pub use boundary::Address;
pub use hooks::{ByKind, Kind, Watch};

pub type Watching<'run> = dyn for<'ast> Watch<'ast> + 'run;

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

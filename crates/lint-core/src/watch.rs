mod boundary;
mod facts;
mod hooks;

use algebra::{Monoid, Semigroup};
use newtype::Arbitrary;

use crate::walk::Walk;

pub use boundary::Address;
pub use facts::Facts;
pub use hooks::{Contribute, Kind};

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Descend {
    #[default]
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
    pub fn entered(self) -> Option<()> {
        (self == Descend::Into).then_some(())
    }
}

impl Semigroup for Descend {
    fn combine(self, next: Descend) -> Descend {
        self.entered().map_or(self, |()| next)
    }
}

impl Monoid for Descend {}

impl Descend {
    pub(crate) fn descending<Node>(self, node: Node) -> Option<Node> {
        matches!(self, Descend::Into | Descend::Unfold).then_some(node)
    }

    pub(crate) fn expanding<Invocation>(self, invocation: Option<Invocation>) -> Option<Invocation> {
        invocation.filter(|_| matches!(self, Descend::Expand | Descend::Unfold))
    }
}

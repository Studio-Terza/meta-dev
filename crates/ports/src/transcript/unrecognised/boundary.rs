use invariants::{invariant, invariants};

use super::Unrecognised;
use crate::transcript::Kind;

invariants! {
    enum Invariant {
        UnreadableIsAKind => "the unreadable kind is a fixed nonempty word, which every Kind check accepts",
    }
}

impl Unrecognised {
    pub(crate) fn unreadable() -> Unrecognised {
        Unrecognised {
            kind: invariant!(holding Kind::try_new(Self::UNREADABLE.to_owned()), Invariant::UnreadableIsAKind),
            uuid: None,
            parent: None,
        }
    }
}

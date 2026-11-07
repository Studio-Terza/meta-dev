use super::binding::Binding;
use super::entries::Admission;
use super::resolver::{Outer, Resolution};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Found<'rows> {
    #[default]
    Nothing,
    One(&'rows Binding),
    Conflict,
}

impl<'rows> Found<'rows> {
    fn union(self, other: Self) -> Self {
        match self {
            Found::Nothing => other,
            Found::One(left) => other.met(left),
            Found::Conflict => Found::Conflict,
        }
    }

    fn met(self, left: &'rows Binding) -> Self {
        match self {
            Found::Nothing => Found::One(left),
            Found::One(right) => Found::agreed(left, right),
            Found::Conflict => Found::Conflict,
        }
    }

    fn undisputed(self) -> Self {
        match self {
            Found::Conflict => Found::Nothing,
            Found::Nothing | Found::One(_) => self,
        }
    }

    fn agreed(left: &'rows Binding, right: &'rows Binding) -> Self {
        match [left].into_iter().find(|binding| *binding == right) {
            Some(agreed) => Found::One(agreed),
            None => Found::Conflict,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Determinacy {
    #[default]
    Determined,
    Weak,
    Strong,
}

impl Determinacy {
    fn or_else<'rows>(self, next: impl FnOnce() -> View<'rows>) -> View<'rows> {
        match self {
            Self::Determined => next(),
            Self::Weak => View::UNEXPANDED.union(next()),
            Self::Strong => View::WAITING,
        }
    }

    fn bound(self, binding: &Binding) -> Resolution {
        match self {
            Self::Determined => Resolution::Bound(binding.clone()),
            Self::Weak | Self::Strong => Resolution::Waiting,
        }
    }

    fn deferring<'rows>(self, disputed: View<'rows>, deferred: impl FnOnce() -> View<'rows>) -> View<'rows> {
        match self {
            Self::Determined => disputed,
            Self::Weak | Self::Strong => deferred(),
        }
    }

    fn absent(self, absent: impl FnOnce() -> Outer) -> Resolution {
        match self {
            Self::Determined => Resolution::Bound(absent().binding()),
            Self::Weak => absent().known(),
            Self::Strong => Resolution::Waiting,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct View<'rows> {
    found: Found<'rows>,
    pending: Determinacy,
}

impl View<'_> {
    pub(super) const ABSENT: View<'static> = View { found: Found::Nothing, pending: Determinacy::Determined };
    pub(super) const WAITING: View<'static> = View { found: Found::Nothing, pending: Determinacy::Strong };
    pub(super) const UNEXPANDED: View<'static> = View { found: Found::Nothing, pending: Determinacy::Weak };
    pub(super) const CONFLICT: View<'static> = View { found: Found::Conflict, pending: Determinacy::Determined };

    pub(super) fn union(self, other: Self) -> Self {
        View { found: self.found.union(other.found), pending: self.pending.max(other.pending) }
    }

    pub(super) fn undisputed(self) -> Self {
        View { found: self.found.undisputed(), pending: self.pending }
    }

    pub(super) fn disputed(self, explicit: impl FnOnce() -> Self, deferred: impl FnOnce() -> Self) -> Self {
        match self.found {
            Found::Conflict => explicit().deferring(self, deferred),
            Found::Nothing | Found::One(_) => self,
        }
    }

    fn deferring(self, disputed: Self, deferred: impl FnOnce() -> Self) -> Self {
        match self.found {
            Found::One(_) => deferred(),
            Found::Conflict => disputed,
            Found::Nothing => self.pending.deferring(disputed, deferred),
        }
    }

    pub(super) fn admitted(admission: Admission, shown: impl FnOnce() -> Self) -> Self {
        match admission {
            Admission::Hidden => View::ABSENT,
            Admission::Shown => shown(),
            Admission::Obscured => View::CONFLICT,
        }
    }

    pub(super) fn or_else(self, next: impl FnOnce() -> Self) -> Self {
        match self.found {
            Found::Nothing => self.pending.or_else(next),
            Found::One(_) | Found::Conflict => self,
        }
    }

    pub(super) fn settled(self, absent: impl FnOnce() -> Outer) -> Resolution {
        match self.found {
            Found::Nothing => self.pending.absent(absent),
            Found::One(binding) => self.pending.bound(binding),
            Found::Conflict => Resolution::Bound(Binding::Unknown),
        }
    }
}

impl<'rows> From<&'rows Binding> for View<'rows> {
    fn from(binding: &'rows Binding) -> Self {
        View { found: Found::One(binding), pending: Determinacy::Determined }
    }
}

use algebra::{Monoid, Semigroup};
use graph::Unresolved;

use super::binding::Binding;
use super::entries::Admission;
use super::resolver::{Outer, Resolution};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Found<'rows> {
    #[default]
    Nothing,
    One(&'rows Binding),
    Conflict,
}

impl Semigroup for Found<'_> {
    fn combine(self, other: Self) -> Self {
        match self {
            Found::Nothing => other,
            Found::One(left) => other.bindings().map_or(Found::Conflict, |right| Found::agreeing(left, right)),
            Found::Conflict => Found::Conflict,
        }
    }
}

impl Monoid for Found<'_> {}

impl<'rows> Found<'rows> {
    fn undisputed(self) -> Found<'rows> {
        match self {
            Found::Conflict => Found::Nothing,
            Found::Nothing | Found::One(_) => self,
        }
    }

    fn bindings(self) -> Result<Option<&'rows Binding>, Conflicted> {
        match self {
            Found::Nothing => Ok(None),
            Found::One(binding) => Ok(Some(binding)),
            Found::Conflict => Err(Conflicted),
        }
    }

    fn agreeing(left: &'rows Binding, right: Option<&'rows Binding>) -> Found<'rows> {
        let agreed = right.map_or(Some(left), |found| Some(left).filter(|binding| *binding == found));
        agreed.map_or(Found::Conflict, Found::One)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Conflicted;

#[derive(Clone, Copy, Debug)]
pub(super) struct Deferral;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Determinacy {
    #[default]
    Determined,
    Weak,
    Strong,
}

impl Determinacy {
    fn bound(self, binding: &Binding) -> Resolution {
        match self {
            Determinacy::Determined => Resolution::Bound(binding.clone()),
            Determinacy::Weak | Determinacy::Strong => Resolution::Waiting,
        }
    }

    fn deferral(self) -> Option<Deferral> {
        match self {
            Determinacy::Determined => None,
            Determinacy::Weak | Determinacy::Strong => Some(Deferral),
        }
    }

    fn absent(self, absent: impl FnOnce() -> Outer) -> Resolution {
        match self {
            Determinacy::Determined => Resolution::Bound(absent().binding()),
            Determinacy::Weak => absent().known(),
            Determinacy::Strong => Resolution::Waiting,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct View<'rows> {
    found: Found<'rows>,
    pending: Determinacy,
}

impl Semigroup for View<'_> {
    fn combine(self, other: Self) -> Self {
        View { found: self.found.combine(other.found), pending: self.pending.max(other.pending) }
    }
}

impl Monoid for View<'_> {}

impl<'rows> View<'rows> {
    pub(super) const ABSENT: View<'static> = View { found: Found::Nothing, pending: Determinacy::Determined };
    pub(super) const WAITING: View<'static> = View { found: Found::Nothing, pending: Determinacy::Strong };
    pub(super) const UNEXPANDED: View<'static> = View { found: Found::Nothing, pending: Determinacy::Weak };
    pub(super) const CONFLICT: View<'static> = View { found: Found::Conflict, pending: Determinacy::Determined };

    pub(super) fn undisputed(self) -> View<'rows> {
        View { found: self.found.undisputed(), pending: self.pending }
    }

    pub(super) fn conflict(self) -> Option<Conflicted> {
        self.found.bindings().err()
    }

    pub(super) fn deferral(self) -> Option<Deferral> {
        match self.found {
            Found::One(_) => Some(Deferral),
            Found::Conflict => None,
            Found::Nothing => self.pending.deferral(),
        }
    }

    pub(super) fn admitted(admission: Admission, shown: impl FnOnce() -> View<'rows>) -> View<'rows> {
        match admission {
            Admission::Hidden => View::ABSENT,
            Admission::Shown => shown(),
            Admission::Obscured => View::CONFLICT,
        }
    }

    pub(super) fn contested(
        self,
        builtin: impl FnOnce() -> Option<Binding>,
        absent: impl FnOnce() -> Outer,
        expanded: impl FnOnce() -> Unresolved,
    ) -> Resolution {
        match self.found {
            Found::Conflict => builtin().map_or_else(|| self.settled(absent, expanded), Resolution::Bound),
            Found::Nothing | Found::One(_) => self.settled(absent, expanded),
        }
    }

    pub(super) fn settled(self, absent: impl FnOnce() -> Outer, expanded: impl FnOnce() -> Unresolved) -> Resolution {
        match self.found {
            Found::Nothing => self.pending.absent(absent),
            Found::One(binding) => self.pending.bound(binding),
            Found::Conflict => Resolution::Bound(Binding::Unknown(absent().unresolved(expanded))),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Nearest<'rows>(View<'rows>);

impl Semigroup for Nearest<'_> {
    fn combine(self, next: Self) -> Self {
        match self.0.found {
            Found::Nothing => Some(self)
                .filter(|held| held.0.pending == Determinacy::Strong)
                .unwrap_or_else(|| Nearest(self.0.combine(next.0))),
            Found::One(_) | Found::Conflict => self,
        }
    }
}

impl Monoid for Nearest<'_> {}

impl<'rows> From<View<'rows>> for Nearest<'rows> {
    fn from(view: View<'rows>) -> Nearest<'rows> {
        Nearest(view)
    }
}

impl<'rows> From<Nearest<'rows>> for View<'rows> {
    fn from(nearest: Nearest<'rows>) -> View<'rows> {
        nearest.0
    }
}

impl<'rows> From<&'rows Binding> for View<'rows> {
    fn from(binding: &'rows Binding) -> View<'rows> {
        View { found: Found::One(binding), pending: Determinacy::Determined }
    }
}

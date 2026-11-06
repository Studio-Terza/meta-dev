use code_model::{ItemPath, Resolved};
use lint_layout::Name;

use super::Namespace;
use super::boundary::ScopeId;
use super::entries::{Admission, ItemId, Via};
use super::resolver::Walk;
use super::std_names::{Canonical, StdNames};
use super::use_path::UsePath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Binding {
    Module(ScopeId),
    Item(ScopeId, ItemId),
    Std(ItemPath),
    External(ItemPath),
    Unknown,
}

impl Binding {
    pub(super) fn stepped(self, segment: &Name, walk: &Walk<'_>) -> Binding {
        match self {
            Binding::Module(module) => walk.entered(module, segment),
            Binding::Std(path) => Binding::Std(path.joined(segment)),
            Binding::External(path) => Binding::External(path.joined(segment)),
            Binding::Item(..) | Binding::Unknown => Binding::Unknown,
        }
    }

    pub(super) fn globbed(self, name: &Name, walk: &Walk<'_>) -> Globbed {
        match self {
            Binding::Module(module) => walk.member(module, name, Via::Glob),
            Binding::Std(path) => StdNames::globbed(&path, name),
            Binding::Item(..) | Binding::External(_) | Binding::Unknown => Globbed::Opaque,
        }
    }

    pub(super) fn resolved(self, space: &Namespace, written: &UsePath) -> Resolved {
        match self {
            Binding::Module(module) => Binding::crate_or_unknown(space.located(module), written),
            Binding::Item(scope, item) => Binding::crate_or_unknown(space.declared_path(scope, item), written),
            Binding::Std(path) => Resolved::Std(path),
            Binding::External(path) => Resolved::External(path),
            Binding::Unknown => Resolved::Unknown(written.written()),
        }
    }

    fn crate_or_unknown(found: Option<ItemPath>, written: &UsePath) -> Resolved {
        match found {
            Some(path) => Resolved::Crate(path),
            None => Resolved::Unknown(written.written()),
        }
    }

    fn agreed(self, other: &Binding) -> Binding {
        [self].into_iter().find(|binding| binding == other).unwrap_or(Binding::Unknown)
    }
}

impl From<Option<&Canonical>> for Binding {
    fn from(canonical: Option<&Canonical>) -> Binding {
        match canonical {
            Some(found) => Binding::Std(found.path()),
            None => Binding::Unknown,
        }
    }
}

impl From<Option<ScopeId>> for Binding {
    fn from(module: Option<ScopeId>) -> Binding {
        match module {
            Some(module) => Binding::Module(module),
            None => Binding::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Globbed {
    Absent,
    Present(Binding),
    Opaque,
}

impl Globbed {
    pub(super) fn joined(self, other: Globbed) -> Globbed {
        match self {
            Globbed::Absent => other,
            Globbed::Present(left) => other.met(left),
            Globbed::Opaque => Globbed::Opaque,
        }
    }

    pub(super) fn admitted(admission: Admission, shown: impl FnOnce() -> Globbed) -> Globbed {
        match admission {
            Admission::Hidden => Globbed::Absent,
            Admission::Shown => shown(),
            Admission::Obscured => Globbed::Opaque,
        }
    }

    pub(super) fn or_else(self, next: impl FnOnce() -> Globbed) -> Globbed {
        match self {
            Globbed::Absent => next(),
            found @ (Globbed::Present(_) | Globbed::Opaque) => found,
        }
    }

    pub(super) fn settled(self, absent: Binding) -> Binding {
        match self {
            Globbed::Absent => absent,
            Globbed::Present(binding) => binding,
            Globbed::Opaque => Binding::Unknown,
        }
    }

    fn met(self, left: Binding) -> Globbed {
        match self {
            Globbed::Absent => Globbed::Present(left),
            Globbed::Present(right) => Globbed::Present(left.agreed(&right)),
            Globbed::Opaque => Globbed::Opaque,
        }
    }
}

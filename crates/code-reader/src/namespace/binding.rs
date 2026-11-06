use code_model::{ItemPath, Resolved};
use lint_layout::Name;

use super::Namespace;
use super::boundary::ScopeId;
use super::resolver::Walk;
use super::std_names::StdNames;
use super::use_path::UsePath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Binding {
    Module(ScopeId),
    Item(ScopeId, Name),
    Std(ItemPath),
    External(ItemPath),
    Unknown,
}

impl Binding {
    pub(super) fn stepped(self, segment: &Name, walk: &Walk<'_>) -> Binding {
        match self {
            Binding::Module(module) => walk.entered(module, segment),
            Binding::Std(path) => Binding::Std(path.as_ref().iter().cloned().chain([segment.clone()]).collect()),
            Binding::External(path) => {
                Binding::External(path.as_ref().iter().cloned().chain([segment.clone()]).collect())
            }
            Binding::Item(..) | Binding::Unknown => Binding::Unknown,
        }
    }

    pub(super) fn globbed(self, name: &Name, walk: &Walk<'_>) -> Globbed {
        match self {
            Binding::Module(module) => walk.member(module, name),
            Binding::Std(path) => StdNames::globbed(&path, name),
            Binding::Item(..) | Binding::External(_) | Binding::Unknown => Globbed::Opaque,
        }
    }

    pub(super) fn resolved(self, space: &Namespace, written: &UsePath) -> Resolved {
        match self {
            Binding::Module(module) => Resolved::Crate(space.located(module)),
            Binding::Item(scope, name) => {
                Resolved::Crate(space.located(scope).as_ref().iter().cloned().chain([name]).collect())
            }
            Binding::Std(path) => Resolved::Std(path),
            Binding::External(path) => Resolved::External(path),
            Binding::Unknown => Resolved::Unknown(written.written()),
        }
    }

    fn agreed(self, other: &Binding) -> Binding {
        [self].into_iter().find(|binding| binding == other).unwrap_or(Binding::Unknown)
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

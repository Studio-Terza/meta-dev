use code_model::{External, ItemId, ItemPath, Primitive, Resolved, Unresolved};
use lint_layout::{Dependencies, Name};

use super::boundary::ScopeId;
use super::entries::{DeclaredId, Sight};
use super::items::ItemIds;
use super::resolver::Resolution;
use super::settling::Target;
use super::std_names::StdNames;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Binding {
    Module(ScopeId),
    Item(ScopeId, DeclaredId),
    Std(ItemPath),
    External(External),
    Primitive(Primitive),
    Unknown(Unresolved),
    Absent,
}

impl Binding {
    pub(super) fn stepped(self, segment: &Name, entered: impl FnOnce(ScopeId) -> Resolution) -> Resolution {
        match self {
            Binding::Module(module) => entered(module),
            Binding::Std(path) => Resolution::Bound(Binding::Std(path.joined(segment))),
            Binding::External(external) => Resolution::Bound(Binding::External(external.joined(segment))),
            Binding::Unknown(cause) => Resolution::Bound(Binding::Unknown(cause)),
            Binding::Item(..) | Binding::Primitive(_) | Binding::Absent => Resolution::Bound(Binding::UNBOUND),
        }
    }

    pub(super) fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Binding::Module(module) => Target::Module(module, sight_of(module)),
            Binding::Std(path) => Target::listed(&path),
            Binding::Item(..)
            | Binding::External(_)
            | Binding::Primitive(_)
            | Binding::Unknown(_)
            | Binding::Absent => Target::Opaque,
        }
    }

    pub(super) fn agreed(&self, found: Binding) -> Binding {
        match [found].into_iter().find(|candidate| candidate == self) {
            Some(agreed) => agreed,
            None => Binding::UNBOUND,
        }
    }

    pub(super) fn resolved(self, ids: &ItemIds, written: ItemPath) -> Resolved {
        match self {
            Binding::Item(scope, item) => Binding::issued(ids.issued(scope, item), written),
            Binding::Std(path) => StdNames::resolved(path),
            Binding::External(external) => Resolved::External(external),
            Binding::Primitive(primitive) => Resolved::Primitive(primitive),
            Binding::Unknown(cause) => Resolved::Unknown(written, cause),
            Binding::Module(_) | Binding::Absent => Resolved::Unknown(written, Unresolved::Unbound),
        }
    }
}

impl Binding {
    pub(super) const UNBOUND: Binding = Binding::Unknown(Unresolved::Unbound);

    pub(super) fn external(first: &Name, dependencies: Option<&Dependencies>) -> Binding {
        let package = dependencies.and_then(|declared| declared.package(first)).cloned();
        Binding::External(External::new(package, ItemPath::default().joined(first)))
    }

    fn issued(item: Option<ItemId>, written: ItemPath) -> Resolved {
        match item {
            Some(issued) => Resolved::Crate(issued),
            None => Resolved::Unknown(written, Unresolved::Unbound),
        }
    }
}

impl From<Option<ScopeId>> for Binding {
    fn from(module: Option<ScopeId>) -> Binding {
        match module {
            Some(scope) => Binding::Module(scope),
            None => Binding::UNBOUND,
        }
    }
}

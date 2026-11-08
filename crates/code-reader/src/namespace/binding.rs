use code_model::{ItemId, ItemPath, Primitive, Resolved};
use lint_layout::Name;

use super::boundary::ScopeId;
use super::entries::{DeclaredId, Sight};
use super::items::ItemIds;
use super::resolver::Resolution;
use super::settling::Target;
use super::std_names::StdNames;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Binding {
    Module(ScopeId),
    Item(ScopeId, DeclaredId),
    Std(ItemPath),
    External(ItemPath),
    Primitive(Primitive),
    Unknown,
}

impl Binding {
    pub(super) fn stepped(self, segment: &Name, entered: impl FnOnce(ScopeId) -> Resolution) -> Resolution {
        match self {
            Binding::Module(module) => entered(module),
            Binding::Std(path) => Resolution::Bound(Binding::Std(path.joined(segment))),
            Binding::External(path) => Resolution::Bound(Binding::External(path.joined(segment))),
            Binding::Item(..) | Binding::Primitive(_) | Binding::Unknown => Resolution::Bound(Binding::Unknown),
        }
    }

    pub(super) fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Binding::Module(module) => Target::Module(module, sight_of(module)),
            Binding::Std(path) => Target::listed(&path),
            Binding::Item(..) | Binding::External(_) | Binding::Primitive(_) | Binding::Unknown => Target::Opaque,
        }
    }

    pub(super) fn agreed(&self, found: Binding) -> Binding {
        match [found].into_iter().find(|candidate| candidate == self) {
            Some(agreed) => agreed,
            None => Binding::Unknown,
        }
    }

    pub(super) fn resolved(self, ids: &ItemIds, written: ItemPath) -> Resolved {
        match self {
            Binding::Item(scope, item) => Binding::issued(ids.issued(scope, item), written),
            Binding::Std(path) => StdNames::resolved(path),
            Binding::External(path) => Resolved::External(path),
            Binding::Primitive(primitive) => Resolved::Primitive(primitive),
            Binding::Module(_) | Binding::Unknown => Resolved::Unknown(written),
        }
    }
}

impl Binding {
    fn issued(item: Option<ItemId>, written: ItemPath) -> Resolved {
        match item {
            Some(issued) => Resolved::Crate(issued),
            None => Resolved::Unknown(written),
        }
    }
}

impl From<Option<ScopeId>> for Binding {
    fn from(module: Option<ScopeId>) -> Binding {
        match module {
            Some(scope) => Binding::Module(scope),
            None => Binding::Unknown,
        }
    }
}

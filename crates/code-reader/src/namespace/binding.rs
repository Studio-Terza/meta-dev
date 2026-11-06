use code_model::{ItemPath, Resolved};
use lint_layout::Name;

use super::boundary::ScopeId;
use super::entries::{ItemId, Sight};
use super::resolver::Resolution;
use super::settling::Target;

pub(super) trait Locating {
    fn located(&self, scope: ScopeId) -> Option<ItemPath>;

    fn declared_path(&self, scope: ScopeId, item: ItemId) -> Option<ItemPath>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Binding {
    Module(ScopeId),
    Item(ScopeId, ItemId),
    Std(ItemPath),
    External(ItemPath),
    Unknown,
}

impl Binding {
    pub(super) fn stepped(self, segment: &Name, entered: impl FnOnce(ScopeId) -> Resolution) -> Resolution {
        match self {
            Binding::Module(module) => entered(module),
            Binding::Std(path) => Resolution::Bound(Binding::Std(path.joined(segment))),
            Binding::External(path) => Resolution::Bound(Binding::External(path.joined(segment))),
            Binding::Item(..) | Binding::Unknown => Resolution::Bound(Binding::Unknown),
        }
    }

    pub(super) fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Binding::Module(module) => Target::Module(module, sight_of(module)),
            Binding::Std(path) => Target::listed(&path),
            Binding::Item(..) | Binding::External(_) | Binding::Unknown => Target::Opaque,
        }
    }

    pub(super) fn resolved(self, places: &impl Locating, written: ItemPath) -> Resolved {
        match self {
            Binding::Module(module) => Binding::crate_or_unknown(places.located(module), written),
            Binding::Item(scope, item) => Binding::crate_or_unknown(places.declared_path(scope, item), written),
            Binding::Std(path) => Resolved::Std(path),
            Binding::External(path) => Resolved::External(path),
            Binding::Unknown => Resolved::Unknown(written),
        }
    }

    fn crate_or_unknown(found: Option<ItemPath>, written: ItemPath) -> Resolved {
        match found {
            Some(path) => Resolved::Crate(path),
            None => Resolved::Unknown(written),
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

use code_model::{Domain, External, ItemId, ItemPath, Primitive, Resolved};
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
    External(External),
    Primitive(Primitive),
    Unknown,
    Absent,
}

impl Binding {
    pub(super) fn stepped(
        self,
        segment: &Name,
        domain: Domain,
        entered: impl FnOnce(ScopeId) -> Resolution,
    ) -> Resolution {
        match self {
            Binding::Module(module) => entered(module),
            Binding::Std(path) => Resolution::Bound(Binding::Std(path.joined(segment))),
            Binding::External(external) => Resolution::Bound(Binding::reached(&external, segment, domain)),
            Binding::Item(..) | Binding::Primitive(_) | Binding::Unknown | Binding::Absent => {
                Resolution::Bound(Binding::Unknown)
            }
        }
    }

    pub(super) fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Binding::Module(module) => Target::Module(module, sight_of(module)),
            Binding::Std(path) => Target::listed(&path),
            Binding::Item(..) | Binding::External(_) | Binding::Primitive(_) | Binding::Unknown | Binding::Absent => {
                Target::Opaque
            }
        }
    }

    pub(super) fn agreed(&self, found: Binding) -> Binding {
        match [found].into_iter().find(|found| found == self) {
            Some(agreed) => agreed,
            None => Binding::Unknown,
        }
    }

    pub(super) fn resolved(self, ids: &ItemIds, written: ItemPath) -> Resolved {
        match self {
            Binding::Item(scope, item) => Binding::issued(ids.issued(scope, item), written),
            Binding::Std(path) => StdNames::resolved(path),
            Binding::External(external) => Resolved::External(external),
            Binding::Primitive(primitive) => Resolved::Primitive(primitive),
            Binding::Module(_) | Binding::Unknown | Binding::Absent => Resolved::Unknown(written),
        }
    }
}

impl Binding {
    fn reached(module: &External, segment: &Name, domain: Domain) -> Binding {
        match domain {
            Domain::Types => Binding::External(module.joined(segment)),
            Domain::Macros => Binding::macro_at(module, segment),
        }
    }

    fn macro_at(module: &External, segment: &Name) -> Binding {
        match module.path().as_ref().split_first().and_then(|(_, inner)| inner.first()) {
            Some(_) => Binding::Unknown,
            None => Binding::External(module.joined(segment)),
        }
    }

    fn issued(item: Option<ItemId>, written: ItemPath) -> Resolved {
        match item {
            Some(item) => Resolved::Crate(item),
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

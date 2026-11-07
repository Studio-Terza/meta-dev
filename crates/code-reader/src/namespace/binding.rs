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
            Self::Module(module) => entered(module),
            Self::Std(path) => Resolution::Bound(Self::Std(path.joined(segment))),
            Self::External(path) => Resolution::Bound(Self::External(path.joined(segment))),
            Self::Item(..) | Self::Primitive(_) | Self::Unknown => Resolution::Bound(Self::Unknown),
        }
    }

    pub(super) fn target(self, sight_of: impl FnOnce(ScopeId) -> Sight) -> Target {
        match self {
            Self::Module(module) => Target::Module(module, sight_of(module)),
            Self::Std(path) => Target::listed(&path),
            Self::Item(..) | Self::External(_) | Self::Primitive(_) | Self::Unknown => Target::Opaque,
        }
    }

    pub(super) fn agreed(&self, found: Self) -> Self {
        match [found].into_iter().find(|found| found == self) {
            Some(agreed) => agreed,
            None => Self::Unknown,
        }
    }

    pub(super) fn resolved(self, ids: &ItemIds, written: ItemPath) -> Resolved {
        match self {
            Self::Item(scope, item) => Self::issued(ids.issued(scope, item), written),
            Self::Std(path) => StdNames::resolved(path),
            Self::External(path) => Resolved::External(path),
            Self::Primitive(primitive) => Resolved::Primitive(primitive),
            Self::Module(_) | Self::Unknown => Resolved::Unknown(written),
        }
    }
}

impl Binding {
    fn issued(item: Option<ItemId>, written: ItemPath) -> Resolved {
        match item {
            Some(item) => Resolved::Crate(item),
            None => Resolved::Unknown(written),
        }
    }
}

impl From<Option<ScopeId>> for Binding {
    fn from(module: Option<ScopeId>) -> Self {
        match module {
            Some(module) => Self::Module(module),
            None => Self::Unknown,
        }
    }
}

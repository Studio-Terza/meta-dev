use std::collections::BTreeSet;

use algebra::Semigroup;
use graph::{ItemPath, Reexport};
use lint_layout::{Name, Whole, Word};
use strum::{IntoStaticStr, VariantArray};

use super::boundary::ScopeId;
use super::entries::Visibility;

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(super) enum Anchor {
    Crate,
    #[strum(serialize = "self")]
    Module,
    Super,
}

impl Anchor {
    pub(super) fn spelled(first: &Name) -> Option<Anchor> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|anchor| first.matches::<Whole>(&[Word::new(<&str>::from(*anchor))]).is_some())
    }

    pub(super) fn reexported(self, written: &ItemPath, declared: &BTreeSet<&Name>) -> Reexport {
        match self {
            Anchor::Crate | Anchor::Module => Reexport::own(written.as_ref().get(1..).unwrap_or_default(), declared),
            Anchor::Super => Reexport::Opaque,
        }
    }

    pub(super) fn anchored(self, anchors: Anchors) -> Option<ScopeId> {
        match self {
            Anchor::Crate => anchors.root,
            Anchor::Module => anchors.module,
            Anchor::Super => anchors.above,
        }
    }
}

impl From<Anchor> for Visibility {
    fn from(anchor: Anchor) -> Visibility {
        match anchor {
            Anchor::Crate => Visibility::Visible,
            Anchor::Module => Visibility::Private,
            Anchor::Super => Visibility::Restricted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Anchors {
    module: Option<ScopeId>,
    above: Option<ScopeId>,
    root: Option<ScopeId>,
}

impl From<ScopeId> for Anchors {
    fn from(module: ScopeId) -> Anchors {
        Anchors { module: Some(module), above: None, root: Some(module) }
    }
}

impl Semigroup for Anchors {
    fn combine(self, inner: Anchors) -> Anchors {
        Anchors {
            module: inner.module.or(self.module),
            above: inner.above.or(inner.module.and(self.module)).or(self.above),
            root: self.root,
        }
    }
}

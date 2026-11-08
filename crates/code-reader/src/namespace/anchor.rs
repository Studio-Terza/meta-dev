use std::collections::BTreeSet;

use code_model::{ItemPath, Reexport};
use lint_layout::{Name, Whole, Word};

use super::boundary::ScopeId;
use super::entries::Visibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Anchor {
    Crate,
    Module,
    Super,
}

impl Anchor {
    const WORDS: [Word<'static>; 3] = [Word::new("crate"), Word::new("self"), Word::new("super")];
    const ANCHORS: [Anchor; 3] = [Anchor::Crate, Anchor::Module, Anchor::Super];

    pub(super) fn spelled(first: &Name) -> Option<Anchor> {
        Self::WORDS
            .iter()
            .zip(Self::ANCHORS)
            .find_map(|(word, anchor)| first.matches::<Whole>(&[*word]).map(|_| anchor))
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

impl Anchors {
    pub(super) fn rooted(root: ScopeId) -> Anchors {
        Anchors { module: Some(root), above: None, root: Some(root) }
    }

    pub(super) fn nested(self, module: ScopeId) -> Anchors {
        Anchors { module: Some(module), above: self.module, root: self.root }
    }
}

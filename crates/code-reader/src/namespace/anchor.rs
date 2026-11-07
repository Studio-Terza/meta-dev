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
    const ANCHORS: [Self; 3] = [Self::Crate, Self::Module, Self::Super];

    pub(super) fn spelled(first: &Name) -> Option<Self> {
        Self::WORDS
            .iter()
            .zip(Self::ANCHORS)
            .find_map(|(word, anchor)| first.matches::<Whole>(&[*word]).map(|_| anchor))
    }

    pub(super) fn anchored(self, anchors: Anchors) -> Option<ScopeId> {
        match self {
            Self::Crate => anchors.root,
            Self::Module => anchors.module,
            Self::Super => anchors.above,
        }
    }
}

impl From<Anchor> for Visibility {
    fn from(anchor: Anchor) -> Self {
        match anchor {
            Anchor::Crate => Self::Visible,
            Anchor::Module => Self::Private,
            Anchor::Super => Self::Restricted,
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
    pub(super) fn rooted(root: ScopeId) -> Self {
        Self { module: Some(root), above: None, root: Some(root) }
    }

    pub(super) fn nested(self, module: ScopeId) -> Self {
        Self { module: Some(module), above: self.module, root: self.root }
    }
}

use std::collections::BTreeSet;

use super::{Export, Graphed};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct At(usize);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Indexed<'graph> {
    at: At,
    export: &'graph Export,
}

impl<'graph> Indexed<'graph> {
    pub(super) fn at(self) -> At {
        self.at
    }

    pub(super) fn export(self) -> &'graph Export {
        self.export
    }
}

impl Graphed {
    pub(super) fn indexed(&self) -> impl Iterator<Item = Indexed<'_>> {
        self.exports.iter().enumerate().map(|(at, export)| Indexed { at: At(at), export })
    }

    pub(super) fn at(&self, at: At) -> Option<&Export> {
        self.exports.get(at.0)
    }

    pub(super) fn dead(&self, alive: &BTreeSet<At>) -> impl Iterator<Item = &Export> {
        self.indexed().filter(|indexed| !alive.contains(&indexed.at)).map(Indexed::export)
    }
}

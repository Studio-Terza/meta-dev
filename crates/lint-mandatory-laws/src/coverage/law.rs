use std::collections::BTreeSet;

use lint_layout::Name;

use super::kind::Kind;
use crate::package::Package;
use crate::subject::Subject;

#[derive(Debug)]
pub(crate) struct Law<'file> {
    package: Package<'file>,
    kind: Kind,
    mentions: BTreeSet<Name>,
}

impl<'file> Law<'file> {
    pub(crate) fn new(package: Package<'file>, kind: Kind, mentions: BTreeSet<Name>) -> Law<'file> {
        Law { package, kind, mentions }
    }

    pub(crate) fn naming(&self, subject: &Subject<'_>) -> Option<&Law<'file>> {
        (self.package == subject.package() && self.mentions.contains(subject.owner())).then_some(self)
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn mentions(&self) -> &BTreeSet<Name> {
        &self.mentions
    }
}

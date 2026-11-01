use std::collections::BTreeSet;

use lint_layout::Name;

use super::kind::Kind;
use crate::package::Package;
use crate::subject::Subject;

#[derive(Debug)]
pub(crate) struct Law {
    package: Package,
    kind: Kind,
    mentions: BTreeSet<Name>,
}

impl Law {
    pub(crate) fn new(package: Package, kind: Kind, mentions: BTreeSet<Name>) -> Law {
        Law { package, kind, mentions }
    }

    pub(crate) fn names(&self, subject: &Subject) -> bool {
        self.package == *subject.package() && self.mentions.contains(subject.owner())
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn mentions(&self) -> &BTreeSet<Name> {
        &self.mentions
    }
}

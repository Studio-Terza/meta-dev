use std::collections::BTreeSet;

use lint_layout::Name;

use crate::kind::Kind;
use crate::package::Package;

#[derive(Debug)]
pub struct Law<'file> {
    package: Package<'file>,
    kind: Kind,
    mentions: BTreeSet<Name>,
}

impl<'file> Law<'file> {
    pub(crate) fn new(package: Package<'file>, kind: Kind, mentions: BTreeSet<Name>) -> Law<'file> {
        Law { package, kind, mentions }
    }

    #[must_use]
    pub fn naming(&self, package: Package<'_>, names: &[&Name]) -> Option<&Law<'file>> {
        Some(self)
            .filter(|law| law.package == package)
            .filter(|law| names.iter().all(|name| law.mentions.contains(*name)))
    }

    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    #[must_use]
    pub fn mentions(&self) -> &BTreeSet<Name> {
        &self.mentions
    }
}

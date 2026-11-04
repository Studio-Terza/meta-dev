use std::collections::{BTreeMap, BTreeSet};

use lint_layout::Name;

use crate::kind::Kind;
use crate::package::Package;

#[derive(Debug)]
pub struct Law<'file> {
    package: Package<'file>,
    kind: Kind,
    mentions: BTreeSet<Name>,
    routes: BTreeMap<Name, BTreeSet<Name>>,
}

impl<'file> Law<'file> {
    pub(crate) fn new(
        package: Package<'file>,
        kind: Kind,
        mentions: BTreeSet<Name>,
        routes: BTreeMap<Name, BTreeSet<Name>>,
    ) -> Law<'file> {
        Law { package, kind, mentions, routes }
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
    pub fn routed(&self, owner: &Name, member: &Name) -> Option<&Name> {
        self.routes.get(owner).and_then(|members| members.get(member))
    }

    #[must_use]
    pub fn mentions(&self) -> &BTreeSet<Name> {
        &self.mentions
    }
}

use std::collections::BTreeSet;

use lint_core::Site;
use lint_layout::Name;

use crate::kind::Kind;
use crate::law_path::{LawPath, Route, Side, Unresolved};
use crate::package::Package;

#[derive(Debug)]
pub(crate) struct Origin<'file> {
    package: Package<'file>,
    site: Site<'file>,
    name: Name,
}

impl<'file> Origin<'file> {
    pub(crate) fn new(package: Package<'file>, site: Site<'file>, name: Name) -> Self {
        Origin { package, site, name }
    }

    fn of_package(&self, package: Package<'_>) -> Option<&Self> {
        Some(self).filter(|origin| origin.package == package)
    }

    fn unresolved(&self) -> Unresolved<'file> {
        Unresolved::new(self.site, self.name.clone())
    }
}

#[derive(Debug)]
pub struct Law<'file> {
    origin: Origin<'file>,
    kind: Kind,
    mentions: BTreeSet<Name>,
    routes: Vec<Route>,
}

impl<'file> Law<'file> {
    pub(crate) fn new(origin: Origin<'file>, kind: Kind, mentions: BTreeSet<Name>, routes: Vec<Route>) -> Self {
        Law { origin, kind, mentions, routes }
    }

    #[must_use]
    pub fn naming(&self, package: Package<'_>, names: &[&Name]) -> Option<&Self> {
        self.of_package(package).filter(|law| names.iter().all(|name| law.mentions.contains(*name)))
    }

    #[must_use]
    pub fn of_package(&self, package: Package<'_>) -> Option<&Self> {
        self.origin.of_package(package).and(Some(self))
    }

    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    #[must_use]
    pub fn routed(&self, owner: &Name, member: &Name) -> Option<&LawPath> {
        self.routes.iter().find_map(|route| route.reaching(owner, member))
    }

    #[must_use]
    pub fn routed_on(&self, side: Side, owner: &Name, member: &Name) -> Option<&LawPath> {
        self.routes.iter().filter_map(|route| route.of_side(side)).find_map(|route| route.reaching(owner, member))
    }

    #[must_use]
    pub fn on_side(&self, side: Side) -> Option<&Route> {
        self.routes.iter().find_map(|route| route.of_side(side))
    }

    #[must_use]
    pub fn unresolved(&self) -> Option<Unresolved<'file>> {
        self.routes.iter().find_map(Route::closure).map(|_| self.origin.unresolved())
    }

    #[must_use]
    pub fn mentions(&self) -> &BTreeSet<Name> {
        &self.mentions
    }
}

impl<'file> AsRef<Site<'file>> for Law<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.origin.site
    }
}

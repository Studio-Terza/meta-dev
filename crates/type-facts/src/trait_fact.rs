use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::dependency::Dependency;
use crate::method::Method;

#[derive(Clone, Debug)]
pub struct TraitFact<'file> {
    krate: &'file CrateName,
    name: Name,
    site: Site<'file>,
    dependencies: Vec<Dependency<'file>>,
    methods: Vec<Method>,
}

impl<'file> TraitFact<'file> {
    pub(crate) fn new(
        krate: &'file CrateName,
        name: Name,
        site: Site<'file>,
        dependencies: Vec<Dependency<'file>>,
    ) -> TraitFact<'file> {
        TraitFact { krate, name, site, dependencies, methods: Vec::new() }
    }

    pub(crate) fn with_methods(mut self, methods: Vec<Method>) -> TraitFact<'file> {
        self.methods = methods;
        self
    }
}

impl<'file> AsRef<[Dependency<'file>]> for TraitFact<'file> {
    fn as_ref(&self) -> &[Dependency<'file>] {
        &self.dependencies
    }
}

impl AsRef<CrateName> for TraitFact<'_> {
    fn as_ref(&self) -> &CrateName {
        self.krate
    }
}

impl AsRef<Name> for TraitFact<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for TraitFact<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl AsRef<[Method]> for TraitFact<'_> {
    fn as_ref(&self) -> &[Method] {
        &self.methods
    }
}

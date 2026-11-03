use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{Dependency, Facts, TraitFact, TypeFact};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Declaration<'facts, 'file> {
    krate: &'facts CrateName,
    name: &'facts Name,
    site: Site<'file>,
    dependencies: &'facts [Dependency<'file>],
}

impl<'facts, 'file> Declaration<'facts, 'file> {
    pub(crate) fn all(facts: &'facts Facts<'file>) -> impl Iterator<Item = Declaration<'facts, 'file>> {
        facts.types().iter().map(Declaration::from).chain(facts.traits().iter().map(Declaration::from))
    }

    pub(crate) fn krate(&self) -> &'facts CrateName {
        self.krate
    }

    pub(crate) fn name(&self) -> &'facts Name {
        self.name
    }

    pub(crate) fn dependencies(&self) -> &'facts [Dependency<'file>] {
        self.dependencies
    }
}

impl<'facts, 'file> From<&'facts TypeFact<'file>> for Declaration<'facts, 'file> {
    fn from(found: &'facts TypeFact<'file>) -> Declaration<'facts, 'file> {
        Declaration {
            krate: found.as_ref(),
            name: found.as_ref(),
            site: *AsRef::<Site<'file>>::as_ref(found),
            dependencies: found.dependencies(),
        }
    }
}

impl<'facts, 'file> From<&'facts TraitFact<'file>> for Declaration<'facts, 'file> {
    fn from(found: &'facts TraitFact<'file>) -> Declaration<'facts, 'file> {
        Declaration {
            krate: found.as_ref(),
            name: found.as_ref(),
            site: *AsRef::<Site<'file>>::as_ref(found),
            dependencies: found.dependencies(),
        }
    }
}

impl<'file> AsRef<Site<'file>> for Declaration<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

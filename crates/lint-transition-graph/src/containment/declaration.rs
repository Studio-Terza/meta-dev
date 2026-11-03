use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{Dependency, Facts, Implementation, TraitFact, TypeFact};

use super::member::Member;
use super::resolver::Resolver;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Declaration<'found, 'file> {
    krate: &'found CrateName,
    name: &'found Name,
    site: Site<'file>,
    dependencies: &'found [Dependency<'file>],
    implementations: &'found [Implementation],
}

impl<'found, 'file> Declaration<'found, 'file> {
    pub(crate) fn all(facts: &'found Facts<'file>) -> Vec<Declaration<'found, 'file>> {
        let types = facts.types().iter().map(|found| Declaration::of_type(facts, found));
        types.chain(facts.traits().iter().map(Declaration::of_trait)).collect()
    }

    fn of_type(facts: &'found Facts<'file>, found: &'found TypeFact<'file>) -> Declaration<'found, 'file> {
        Declaration {
            krate: found.as_ref(),
            name: found.as_ref(),
            site: *AsRef::<Site<'file>>::as_ref(found),
            dependencies: found.dependencies(),
            implementations: facts.implementations(found),
        }
    }

    fn of_trait(found: &'found TraitFact<'file>) -> Declaration<'found, 'file> {
        Declaration {
            krate: found.as_ref(),
            name: found.as_ref(),
            site: *AsRef::<Site<'file>>::as_ref(found),
            dependencies: found.dependencies(),
            implementations: &[],
        }
    }

    pub(crate) fn key(&self) -> (&'found CrateName, &'found Name) {
        (self.krate, self.name)
    }

    pub(crate) fn site(&self) -> Site<'file> {
        self.site
    }

    pub(crate) fn placed(&self, segments: &'found [Name]) -> Member<'found> {
        Member::new(self.krate, self.name, segments)
    }

    pub(crate) fn used(&self, resolver: &Resolver<'found>) -> Vec<Member<'found>> {
        let held = self
            .dependencies
            .iter()
            .filter_map(|dependency| resolver.member(self.krate, dependency.as_ref(), dependency.qualifier()));
        let implemented =
            self.implementations.iter().filter_map(|implemented| resolver.member(self.krate, implemented.as_ref(), None));
        held.chain(implemented).collect()
    }
}

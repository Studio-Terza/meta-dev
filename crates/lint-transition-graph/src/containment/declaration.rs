use lint_core::Site;
use lint_placement::CrateName;
use type_facts::{Dependency, Facts, Implementation, TraitFact, TypeFact};

use super::link::Link;
use super::member::Member;
use super::resolver::Resolver;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Declaration<'found, 'file> {
    krate: &'found CrateName,
    named: Member<'found>,
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
            named: Member::declared(found.as_ref(), found.as_ref()),
            site: *AsRef::<Site<'file>>::as_ref(found),
            dependencies: AsRef::<[Dependency<'file>]>::as_ref(found),
            implementations: facts.implementations(found),
        }
    }

    fn of_trait(found: &'found TraitFact<'file>) -> Declaration<'found, 'file> {
        Declaration {
            krate: found.as_ref(),
            named: Member::declared(found.as_ref(), found.as_ref()),
            site: *AsRef::<Site<'file>>::as_ref(found),
            dependencies: AsRef::<[Dependency<'file>]>::as_ref(found),
            implementations: &[],
        }
    }

    pub(crate) fn links(&self, placed: Member<'found>, resolver: &Resolver<'found>) -> Vec<Link<'found>> {
        let held = self
            .dependencies
            .iter()
            .filter_map(|dependency| resolver.member(self.krate, dependency.as_ref(), dependency.qualifier()));
        let implemented = self
            .implementations
            .iter()
            .filter_map(|implemented| resolver.member(self.krate, implemented.as_ref(), None));
        held.chain(implemented).map(|used| Link::new(placed, used)).collect()
    }
}

impl<'found> AsRef<Member<'found>> for Declaration<'found, '_> {
    fn as_ref(&self) -> &Member<'found> {
        &self.named
    }
}

impl<'file> AsRef<Site<'file>> for Declaration<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

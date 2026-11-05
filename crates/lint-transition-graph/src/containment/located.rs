use lint_core::Site;
use type_facts::Outside;

use super::link::Link;
use super::member::Member;
use super::outsider::Outsider;
use super::resolver::Resolver;

#[derive(Debug)]
pub(crate) struct Located<'found, 'file> {
    outside: Outside<'found, 'file>,
    implementers: Vec<Member<'found>>,
    traits: Vec<Member<'found>>,
}

impl<'found, 'file> Located<'found, 'file> {
    pub(crate) fn resolved_by(resolver: &Resolver<'found>, outside: Outside<'found, 'file>) -> Located<'found, 'file> {
        let implementation = outside.implementation();
        let site: &Site<'file> = implementation.as_ref();
        Located {
            outside,
            implementers: resolver.members(site.path(), outside.name(), implementation.target_path().as_ref(), None),
            traits: resolver.members(site.path(), implementation.as_ref(), implementation.written(), None),
        }
    }

    pub(crate) fn links(&self) -> impl Iterator<Item = Link<'found>> + '_ {
        self.implementers.iter().flat_map(|implementer| self.traits.iter().map(|used| Link::new(*implementer, *used)))
    }

    pub(crate) fn outsiders(&self) -> impl Iterator<Item = Outsider<'found, 'file>> + '_ {
        let site: &Site<'file> = self.outside.implementation().as_ref();
        self.traits
            .iter()
            .filter(|_| self.implementers.is_empty())
            .map(|used| Outsider::new(self.outside.name(), *site, *used))
    }
}

mod method;

use std::collections::BTreeSet;
use std::fmt;

use equivalence::{Equivalence, Group, Groups};
use lint_core::Site;
use lint_layout::Name;

use crate::listed::Listed;
use crate::system::TypeSystem;
use method::Method;

#[derive(Debug)]
pub struct Alike<'system, 'found, 'file>(Group<Method<'system, 'found, 'file>>);

impl<'system, 'found, 'file> Alike<'system, 'found, 'file> {
    #[must_use]
    pub fn grouped(
        system: &'system TypeSystem<'found, 'file>,
        equivalence: Equivalence,
    ) -> Vec<Alike<'system, 'found, 'file>> {
        let types = system.types();
        let methods = system.edges().iter().filter_map(|edge| Method::of_edge(types, edge, equivalence));
        Groups::grouped(methods).listed().into_iter().map(Alike).filter_map(Alike::named_apart).collect()
    }

    #[must_use]
    pub fn site(&self) -> Option<Site<'file>> {
        self.0.sites().first().map(|site| **site)
    }

    fn named_apart(self) -> Option<Alike<'system, 'found, 'file>> {
        let names: BTreeSet<&Name> = self.0.members().iter().map(AsRef::as_ref).collect();
        names.iter().nth(1).map(|_| ()).map(|()| self)
    }
}

impl fmt::Display for Alike<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut seen = BTreeSet::new();
        let named = self.0.members().iter().filter(|method| seen.insert(AsRef::<Name>::as_ref(*method)));
        let members = Listed::<_, ','>::joined(named.map(Method::located));
        let first = self.0.members().first();
        first.map_or(Ok(()), |method| {
            write!(formatter, "{}: {members} share the signature {}", method.owner(), self.0.key())
        })
    }
}

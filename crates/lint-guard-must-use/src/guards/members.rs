mod finding;

use std::collections::{BTreeMap, BTreeSet};

use algebra::{Monoid, Semigroup};
use lint_core::Module;
use lint_core::{Site, Violation};
use lint_layout::Name;

use super::type_key::TypeKey;
use finding::Finding;

#[derive(Debug, Default)]
pub struct Members<'file> {
    declared: BTreeMap<Name, BTreeSet<Site<'file>>>,
    aliases: BTreeMap<Name, Name>,
    dropped: Vec<Name>,
}

impl<'file> Members<'file> {
    pub fn declaring(name: Name, unmarked: Option<Site<'file>>) -> Members<'file> {
        Members { declared: BTreeMap::from([(name, unmarked.into_iter().collect())]), ..Members::default() }
    }

    pub fn aliasing(name: Name, target: Option<Name>) -> Members<'file> {
        Members { aliases: target.map(|aliased| (name, aliased)).into_iter().collect(), ..Members::default() }
    }

    pub fn dropping(name: Option<Name>) -> Members<'file> {
        Members { dropped: name.into_iter().collect(), ..Members::default() }
    }

    pub fn declares(&self, name: &Name) -> Option<&Members<'file>> {
        self.declared.contains_key(name).then_some(self)
    }

    pub fn dropped(&self) -> impl Iterator<Item = &Name> {
        self.dropped.iter().map(|name| self.aliases.get(name).unwrap_or(name))
    }

    pub fn unmarked<'members>(
        &'members self,
        module: &'members Module,
        bound: &'members BTreeSet<TypeKey<'members>>,
    ) -> impl Iterator<Item = Violation<'file>> + 'members {
        self.declared
            .iter()
            .filter(move |(name, _)| bound.contains(&TypeKey::new(module, name)))
            .flat_map(|(name, sites)| sites.iter().map(move |site| Violation::new(*site, Finding::from(name))))
    }
}

impl Semigroup for Members<'_> {
    fn combine(self, other: Self) -> Self {
        let declared = other.declared.into_iter().fold(self.declared, |mut declared, (name, sites)| {
            declared.entry(name).or_default().extend(sites);
            declared
        });
        let mut aliases = self.aliases;
        aliases.extend(other.aliases);
        Members { declared, aliases, dropped: self.dropped.combine(other.dropped) }
    }
}

impl Monoid for Members<'_> {}

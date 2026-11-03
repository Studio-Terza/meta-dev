mod finding;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Site, Violation};
use lint_layout::Name;
use lint_paths::Module;

use super::type_key::TypeKey;
use finding::Finding;

#[derive(Debug, Default)]
pub struct Members<'file> {
    declared: BTreeMap<Name, BTreeSet<Site<'file>>>,
    aliases: BTreeMap<Name, Name>,
    dropped: Vec<Name>,
}

impl<'file> Members<'file> {
    pub fn declare(&mut self, name: Name, unmarked: Option<Site<'file>>) {
        self.declared.entry(name).or_default().extend(unmarked);
    }

    pub fn alias(&mut self, name: Name, target: Option<Name>) {
        self.aliases.extend(target.map(|target| (name, target)));
    }

    pub fn drop_implemented(&mut self, name: Option<Name>) {
        self.dropped.extend(name);
    }

    pub fn merged(mut self, other: Members<'file>) -> Members<'file> {
        self.declared = other.declared.into_iter().fold(self.declared, |mut declared, (name, sites)| {
            declared.entry(name).or_default().extend(sites);
            declared
        });
        self.aliases.extend(other.aliases);
        self.dropped.extend(other.dropped);
        self
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

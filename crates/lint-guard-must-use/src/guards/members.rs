mod finding;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::Extended;
use lint_core::Module;
use lint_core::{Site, Violation};
use lint_layout::Name;

use super::type_key::TypeKey;
use finding::Finding;

#[derive(Debug)]
pub struct Dropped(Option<Name>);

impl From<Option<Name>> for Dropped {
    fn from(name: Option<Name>) -> Dropped {
        Dropped(name)
    }
}

#[derive(Debug, Default)]
pub struct Members<'file> {
    declared: BTreeMap<Name, BTreeSet<Site<'file>>>,
    aliases: BTreeMap<Name, Name>,
    dropped: Vec<Name>,
}

impl<'file> Members<'file> {
    pub fn declared(self, name: Name, unmarked: Option<Site<'file>>) -> Members<'file> {
        let mut declared = self.declared;
        declared.entry(name).or_default().extend(unmarked);
        Members { declared, ..self }
    }

    pub fn aliased(self, name: Name, target: Option<Name>) -> Members<'file> {
        Members { aliases: self.aliases.extended(target.map(|aliased| (name, aliased))), ..self }
    }

    pub fn drop_implemented(self, dropped: Dropped) -> Members<'file> {
        Members { dropped: self.dropped.extended(dropped.0), ..self }
    }

    fn merged(mut self, other: Members<'file>) -> Members<'file> {
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

impl<'file> FromIterator<Members<'file>> for Members<'file> {
    fn from_iter<Parts: IntoIterator<Item = Members<'file>>>(parts: Parts) -> Members<'file> {
        parts.into_iter().fold(Members::default(), Members::merged)
    }
}

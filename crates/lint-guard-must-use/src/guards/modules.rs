use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Site, Violation};
use lint_file::Concluded;
use lint_layout::Name;
use lint_paths::Module;
use lint_placement::CrateFile;

use super::members::Members;
use super::type_key::TypeKey;

#[derive(Debug, Default)]
pub struct Modules<'file>(BTreeMap<Module<'file>, Members<'file>>);

impl<'file> FromIterator<Modules<'file>> for Modules<'file> {
    fn from_iter<Parts: IntoIterator<Item = Modules<'file>>>(parts: Parts) -> Modules<'file> {
        parts.into_iter().flat_map(|part| part.0).fold(Modules::default(), |mut merged, (module, members)| {
            merged.file(module, members);
            merged
        })
    }
}

impl<'file> Modules<'file> {
    pub fn file(&mut self, module: Module<'file>, members: Members<'file>) {
        let filed = self.0.remove(&module).unwrap_or_default().merged(members);
        self.0.insert(module, filed);
    }

    pub fn violations(self) -> Vec<Violation<'file>> {
        let bound = self.bound();
        let found: BTreeMap<Site<'file>, Violation<'file>> = self
            .0
            .iter()
            .flat_map(|(module, members)| members.unmarked(module, &bound))
            .map(|violation| (*violation.site(), violation))
            .collect();
        found.into_values().collect()
    }

    fn bound(&self) -> BTreeSet<TypeKey<'_>> {
        self.0
            .iter()
            .flat_map(|(module, members)| members.dropped().flat_map(move |name| self.binding(module, members, name)))
            .collect()
    }

    fn binding<'modules>(
        &'modules self,
        module: &'modules Module<'file>,
        members: &Members<'file>,
        name: &'modules Name,
    ) -> Vec<TypeKey<'modules>> {
        let shared = || {
            let root = <&CrateFile>::from(module);
            self.0
                .iter()
                .filter(|(_, others)| others.declares(name).is_some())
                .filter(|(other, _)| <&CrateFile>::from(*other) == root)
                .map(|(other, _)| TypeKey::new(other, name))
                .collect()
        };
        members.declares(name).map_or_else(shared, |_| vec![TypeKey::new(module, name)])
    }
}

impl<'file> FromIterator<Modules<'file>> for Concluded<'file> {
    fn from_iter<Parts: IntoIterator<Item = Modules<'file>>>(parts: Parts) -> Concluded<'file> {
        parts.into_iter().collect::<Modules<'file>>().violations().into_iter().collect()
    }
}

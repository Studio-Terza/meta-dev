use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{ItemMod, ItemStruct};

use lint_core::{Parsed, Walk};
use lint_layout::Name;
use lint_paths::{Module, Nested};
use lint_placement::CrateName;

use crate::judge::Judge;

#[derive(Debug, Default)]
pub struct Holders(BTreeMap<CrateName, BTreeMap<Name, BTreeSet<Name>>>);

impl Holders {
    pub fn read(parsed: Parsed<'_, '_>, judge: &Judge<'_>) -> Holders {
        let mut reader = Reader { module: Module::of_head(parsed.head()), judge, owners: BTreeMap::new() };
        reader.visit_file(parsed.file().syntax());
        Holders(BTreeMap::from([(CrateName::from(parsed.head().placement().crate_root()), reader.owners)]))
    }

    pub fn fields(&self, crate_name: &CrateName, owner: &Name) -> Option<&BTreeSet<Name>> {
        self.0.get(crate_name)?.get(owner)
    }

    fn absorbed(self, part: Holders) -> Holders {
        let fields = part.0.into_iter().flat_map(|(crate_name, owners)| {
            owners.into_iter().map(move |(owner, held)| (crate_name.clone(), owner, held))
        });
        fields.fold(self, |mut merged, (crate_name, owner, held)| {
            merged.0.entry(crate_name).or_default().entry(owner).or_default().extend(held);
            merged
        })
    }
}

struct Reader<'file, 'run> {
    module: Module<'file>,
    judge: &'run Judge<'run>,
    owners: BTreeMap<Name, BTreeSet<Name>>,
}

impl<'file> Nested<'file> for Reader<'file, '_> {
    fn module(&mut self) -> &mut Module<'file> {
        &mut self.module
    }
}

impl<'ast> Visit<'ast> for Reader<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.nested(module, |reader| visit::visit_item_mod(reader, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || {
            let generics = self.judge.generics(&self.module, &item.generics);
            item.fields
                .iter()
                .filter(|field| self.judge.port_type(&self.module, &generics, &field.ty).is_some())
                .filter_map(|field| field.ident.as_ref())
                .map(Name::from)
                .fold(&mut self.owners, |owners, field| {
                    owners.entry(Name::from(&item.ident)).or_default().insert(field);
                    owners
                });
        });
    }
}

impl FromIterator<Holders> for Holders {
    fn from_iter<Parts: IntoIterator<Item = Holders>>(parts: Parts) -> Holders {
        parts.into_iter().fold(Holders::default(), Holders::absorbed)
    }
}

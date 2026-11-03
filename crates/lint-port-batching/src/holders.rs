use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{ItemMod, ItemStruct};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;
use lint_paths::{Module, Nested};
use lint_placement::CrateName;

use crate::judge::Judge;

#[derive(Debug, Default)]
pub struct Holders(BTreeMap<CrateName, BTreeMap<Name, BTreeSet<Name>>>);

impl Holders {
    pub fn declared(files: &[&SourceFile], judge: &Judge<'_>) -> Holders {
        files.iter().fold(Holders::default(), |mut found, file| {
            let mut reader = Reader { module: Module::of_file(file), judge, owners: BTreeMap::new() };
            reader.visit_file(file.syntax());
            let crate_owners = found.0.entry(CrateName::from(file.placement().crate_root())).or_default();
            reader.owners.into_iter().for_each(|(owner, fields)| crate_owners.entry(owner).or_default().extend(fields));
            found
        })
    }

    pub fn fields(&self, crate_name: &CrateName, owner: &Name) -> Option<&BTreeSet<Name>> {
        self.0.get(crate_name)?.get(owner)
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
            let held: BTreeSet<Name> = item
                .fields
                .iter()
                .filter(|field| self.judge.port_type(&self.module, &generics, &field.ty).is_some())
                .filter_map(|field| field.ident.as_ref())
                .map(Name::from)
                .collect();
            Some(held).filter(|held| !held.is_empty()).into_iter().fold(&mut self.owners, |owners, held| {
                owners.entry(Name::from(&item.ident)).or_default().extend(held);
                owners
            });
        });
    }
}

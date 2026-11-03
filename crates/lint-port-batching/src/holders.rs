use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{ItemMod, ItemStruct};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;
use lint_paths::Module;
use lint_placement::CrateName;

use crate::judge::Judge;

#[derive(Debug, Default)]
pub struct Holders(BTreeSet<Held>);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Held {
    crate_name: CrateName,
    owner: Name,
    field: Name,
}

impl Holders {
    pub fn declared(files: &[&SourceFile], judge: &Judge<'_>) -> Holders {
        files.iter().fold(Holders::default(), |found, file| {
            let mut reader = Reader {
                crate_name: CrateName::from(file.placement().crate_root()),
                module: Module::of_file(file),
                judge,
                found,
            };
            reader.visit_file(file.syntax());
            reader.found
        })
    }

    pub fn fields(&self, crate_name: &CrateName, owner: &Name) -> BTreeSet<Name> {
        self.0
            .iter()
            .filter(|held| held.crate_name == *crate_name && held.owner == *owner)
            .map(|held| held.field.clone())
            .collect()
    }

    pub fn holder(&self, crate_name: &CrateName, owner: &Name) -> Option<Name> {
        self.0.iter().find(|held| held.crate_name == *crate_name && held.owner == *owner).map(|held| held.owner.clone())
    }
}

struct Reader<'run> {
    crate_name: CrateName,
    module: Module,
    judge: &'run Judge<'run>,
    found: Holders,
}

impl<'ast> Visit<'ast> for Reader<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.module.clone();
        self.module = outer.within(&Name::from(&module.ident));
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.module = outer;
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || {
            let generics = self.judge.generics(&self.module, &item.generics);
            let held: Vec<Held> = item
                .fields
                .iter()
                .filter(|field| self.judge.port_type(&self.module, &generics, &field.ty).is_some())
                .filter_map(|field| field.ident.as_ref())
                .map(|field| Held {
                    crate_name: self.crate_name.clone(),
                    owner: Name::from(&item.ident),
                    field: Name::from(field),
                })
                .collect();
            self.found.0.extend(held);
        });
    }
}

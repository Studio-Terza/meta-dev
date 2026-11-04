use std::collections::{BTreeMap, BTreeSet};

use syn::ItemTrait;
use syn::visit::{self, Visit};

use crate::operators::Operators;
use crate::reader::Import;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug, Default)]
pub(crate) struct Traits<'file>(BTreeMap<&'file CrateName, BTreeSet<Name>>);

impl<'file> Traits<'file> {
    const HOME_ROOTS: [&'static str; 3] = ["crate", "self", "super"];

    pub(crate) fn declared(files: &'file [SourceFile]) -> Traits<'file> {
        files.iter().fold(Traits::default(), |traits, file| traits.read(file, file.placement().package().crate_name()))
    }

    pub(crate) fn declaring<'name>(
        &self,
        name: &'name Name,
        import: Option<&Import>,
        home: &CrateName,
    ) -> Option<&'name Name> {
        let imported = import
            .and_then(Import::root)
            .filter(|root| Self::HOME_ROOTS.iter().all(|home_root| **root != *home_root))
            .map(CrateName::from);
        Some(name)
            .filter(|name| {
                self.0.get(imported.as_ref().unwrap_or(home)).is_some_and(|declared| declared.contains(*name))
            })
            .or_else(|| Operators::named(name))
    }

    fn read(mut self, file: &'file SourceFile, home: &'file CrateName) -> Traits<'file> {
        let mut declared = Declared::default();
        declared.visit_file(file.syntax());
        self.0.entry(home).or_default().extend(declared.0);
        file.test_modules().iter().fold(self, |traits, module| traits.read(module, home))
    }
}

#[derive(Debug, Default)]
struct Declared(BTreeSet<Name>);

impl<'ast> Visit<'ast> for Declared {
    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.0.insert(Name::from(&item.ident));
        visit::visit_item_trait(self, item);
    }
}

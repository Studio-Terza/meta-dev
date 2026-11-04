use std::collections::BTreeSet;

use syn::ItemTrait;
use syn::visit::{self, Visit};

use crate::operators::Operators;
use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Traits(BTreeSet<Name>);

impl Traits {
    pub(crate) fn declared(files: &[SourceFile]) -> Traits {
        files.iter().fold(Traits::default(), Traits::read)
    }

    pub(crate) fn declaring<'name>(&self, name: &'name Name) -> Option<&'name Name> {
        Some(name).filter(|name| self.0.contains(*name)).or_else(|| Operators::named(name))
    }

    fn read(mut self, file: &SourceFile) -> Traits {
        self.visit_file(file.syntax());
        file.test_modules().iter().fold(self, Traits::read)
    }
}

impl<'ast> Visit<'ast> for Traits {
    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.0.insert(Name::from(&item.ident));
        visit::visit_item_trait(self, item);
    }
}

mod borrows;
mod derived;
mod finding;

use std::path::Path;

use syn::ItemStruct;
use syn::visit::{self, Visit};

use borrows::Borrows;
use derived::Derived;
use finding::Finding;
use lint_core::{Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name};

#[derive(Debug)]
pub struct Ungenerated<'file> {
    path: &'file Path,
    found: Vec<Violation>,
}

impl<'file> Ungenerated<'file> {
    pub fn in_file(file: &'file SourceFile) -> Vec<Violation> {
        let mut ungenerated = Ungenerated { path: file.path(), found: Vec::new() };
        ungenerated.visit_file(file.syntax());
        ungenerated.found
    }
}

impl<'ast> Visit<'ast> for Ungenerated<'_> {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let derived = Derived::from(item.attrs.as_slice());
        self.found.extend(derived.lacks_generator().and(Borrows::owning(item)).map(|owning| {
            Violation::new(
                Site::new(self.path, LineNumber::from(owning.struct_token.span)),
                Finding::from(Name::from(&owning.ident)),
            )
        }));
        visit::visit_item_struct(self, item);
    }
}

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
        self.found.extend((Derived::from(item.attrs.as_slice()).lacks_generator() && !Borrows::within(item)).then(
            || {
                Violation::new(
                    Site::new(self.path, LineNumber::from(item.struct_token.span)),
                    Finding::from(Name::from(&item.ident)),
                )
            },
        ));
        visit::visit_item_struct(self, item);
    }
}

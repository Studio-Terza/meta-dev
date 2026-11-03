use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{Index, ItemMod, ItemStruct, Visibility};

use super::finding::Finding;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_file::FileLint;
use lint_layout::LineNumber;
use lint_layout::Name;

pub struct FieldVisibility<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl FileLint for FieldVisibility<'_> {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut fields = FieldVisibility { path: file.path(), violations: Vec::new() };
        fields.visit_file(file.syntax());
        fields.violations
    }
}

impl<'ast> Visit<'ast> for FieldVisibility<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || self.exposed(item));
    }
}

impl FieldVisibility<'_> {
    fn exposed(&mut self, item: &ItemStruct) {
        let exposed = item
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| matches!(field.vis, Visibility::Public(_) | Visibility::Restricted(_)))
            .map(|(index, field)| {
                Violation::new(
                    Site::new(self.path, LineNumber::from(field.span())),
                    Finding::ExposedField { field, index: Index::from(index), owner: Name::from(&item.ident) },
                )
            });
        self.violations.extend(exposed);
    }
}

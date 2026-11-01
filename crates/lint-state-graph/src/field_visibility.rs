use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{Index, ItemMod, ItemStruct, Member, Visibility};

use super::finding::Finding;
use lint_core::LineNumber;
use lint_core::Name;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;

pub struct FieldVisibility<'scan> {
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl<'scan> FieldVisibility<'scan> {
    pub fn violations(file: &'scan SourceFile) -> Vec<Violation> {
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
            .filter(|(_, field)| !matches!(field.vis, Visibility::Inherited))
            .map(|(index, field)| {
                Violation::new(
                    Site::new(self.path, LineNumber::from(field.span())),
                    Finding::ExposedField {
                        field: field.ident.clone().map_or_else(|| Member::Unnamed(Index::from(index)), Member::Named),
                        owner: Name::from(&item.ident),
                        visibility: field.vis.clone(),
                    },
                )
            });
        self.violations.extend(exposed);
    }
}

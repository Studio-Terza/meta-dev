mod finding;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Field, Fields, Ident, ItemEnum, ItemMod, ItemStruct};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::LineNumber;

use crate::attributes::Attributes;
use crate::text_mention::TextMention;
use crate::wrappers::Wrappers;
use finding::Finding;

pub struct PlainFields<'scan, 'file> {
    file: &'file SourceFile,
    wrappers: &'scan Wrappers,
    found: Vec<Violation<'file>>,
}

impl<'scan, 'file> PlainFields<'scan, 'file> {
    pub fn in_file(file: &'file SourceFile, wrappers: &'scan Wrappers) -> Vec<Violation<'file>> {
        let mut scan = PlainFields { file, wrappers, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn read_in(attributes: &[Attribute]) -> Walk {
        let attrs = Attributes::flattened_from(attributes);
        Some(Walk::outside_tests(attributes))
            .filter(|walk| *walk == Walk::Entered)
            .and(attrs.deserialize_derive())
            .filter(|_| attrs.validation().is_none())
            .map_or(Walk::Skipped, |_| Walk::Entered)
    }

    fn inspect(&mut self, owner: &Ident, fields: &Fields) {
        let file = self.file;
        let wrappers = self.wrappers;
        self.found.extend(fields.iter().filter(|field| TextMention::in_type(&field.ty, wrappers).is_some()).map(
            |field: &Field| {
                let context = field.ident.as_ref().unwrap_or(owner);
                Violation::new(Site::new(file.path(), LineNumber::from(field.ty.span())), Finding::FreeText { context })
            },
        ));
    }
}

impl<'ast> Visit<'ast> for PlainFields<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        match Self::read_in(&item.attrs) {
            Walk::Entered => self.inspect(&item.ident, &item.fields),
            Walk::Skipped => {}
        }
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        match Self::read_in(&item.attrs) {
            Walk::Entered => {
                item.variants.iter().for_each(|variant| self.inspect(&variant.ident, &variant.fields));
            }
            Walk::Skipped => {}
        }
    }
}

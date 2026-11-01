mod finding;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Field, Fields, ItemEnum, ItemMod, ItemStruct};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name};

use crate::attributes::Attributes;
use crate::text_mention::TextMention;
use crate::wrappers::Wrappers;
use finding::Finding;

pub struct PlainFields<'file> {
    file: &'file SourceFile,
    wrappers: &'file Wrappers,
    found: Vec<Violation>,
}

impl<'file> PlainFields<'file> {
    pub fn in_file(file: &'file SourceFile, wrappers: &'file Wrappers) -> Vec<Violation> {
        let mut scan = PlainFields { file, wrappers, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found
    }

    fn read_in(attributes: &[Attribute]) -> Walk {
        let attrs = Attributes::flattened_from(attributes);
        match (Walk::outside_tests(attributes), attrs.derives_deserialize() && attrs.validation().is_none()) {
            (Walk::Entered, true) => Walk::Entered,
            _ => Walk::Skipped,
        }
    }

    fn inspect(&mut self, owner: &Name, fields: &Fields) {
        let file = self.file;
        let wrappers = self.wrappers;
        self.found.extend(fields.iter().filter(|field| TextMention::in_type(&field.ty, wrappers)).map(
            |field: &Field| {
                let context = field.ident.as_ref().map_or_else(|| owner.clone(), Name::from);
                Violation::new(Site::new(file.path(), LineNumber::from(field.ty.span())), Finding::FreeText { context })
            },
        ));
    }
}

impl<'ast> Visit<'ast> for PlainFields<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        match Self::read_in(&item.attrs) {
            Walk::Entered => self.inspect(&Name::from(&item.ident), &item.fields),
            Walk::Skipped => {}
        }
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        match Self::read_in(&item.attrs) {
            Walk::Entered => {
                item.variants.iter().for_each(|variant| self.inspect(&Name::from(&variant.ident), &variant.fields));
            }
            Walk::Skipped => {}
        }
    }
}

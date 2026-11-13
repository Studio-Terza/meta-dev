use syn::visit::{self, Visit};
use syn::{ItemEnum, ItemImpl, ItemStruct};

mod shape;

use super::profile::Profile;
use lint_core::{FileHead, Parsed};
use shape::Shape;

pub(crate) struct Reader<'file> {
    head: &'file FileHead,
    profiles: Vec<Profile<'file>>,
}

impl<'file> Reader<'file> {
    pub(crate) fn read(parsed: Parsed<'_, 'file>) -> Vec<Profile<'file>> {
        let mut reader = Reader { head: parsed.head(), profiles: Vec::new() };
        reader.visit_file(parsed.file().syntax());
        reader.profiles
    }
}

impl<'ast> Visit<'ast> for Reader<'_> {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.profiles.push(Shape::of_struct(self.head, item));
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.profiles.push(Shape::of_enum(self.head, item));
        visit::visit_item_enum(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.profiles.extend(Shape::of_impl(self.head, item));
        visit::visit_item_impl(self, item);
    }
}

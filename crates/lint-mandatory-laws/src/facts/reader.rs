use syn::visit::{self, Visit};
use syn::{ItemEnum, ItemImpl, ItemStruct};

mod shape;

use super::profile::Profile;
use lint_core::SourceFile;
use shape::Shape;

pub(crate) struct Reader<'file> {
    file: &'file SourceFile,
    profiles: Vec<Profile>,
}

impl<'file> Reader<'file> {
    pub(crate) fn read(file: &'file SourceFile) -> Vec<Profile> {
        let mut reader = Reader { file, profiles: Vec::new() };
        reader.visit_file(file.syntax());
        reader.profiles
    }
}

impl<'ast> Visit<'ast> for Reader<'_> {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.profiles.push(Shape::of_struct(self.file, item));
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.profiles.push(Shape::of_enum(self.file, item));
        visit::visit_item_enum(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.profiles.extend(Shape::of_impl(self.file, item));
        visit::visit_item_impl(self, item);
    }
}

use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{ItemEnum, ItemMod, ItemStruct};

use lint_layout::Name;

use crate::declared_type::DeclaredType;
use crate::home::Home;
use crate::source_file::SourceFile;
use crate::walk::Walk;

#[derive(Debug, Default)]
pub struct TypeHomes<'file> {
    declared: BTreeMap<Name, Vec<Home<'file>>>,
    reading: Home<'file>,
}

impl<'file> TypeHomes<'file> {
    #[must_use]
    pub fn declared_in(files: &[&'file SourceFile]) -> TypeHomes<'file> {
        files.iter().fold(TypeHomes::default(), |homes, file| homes.reading(file))
    }

    #[must_use]
    pub fn resolve(&self, name: Name, from: Home<'file>) -> DeclaredType<'file> {
        DeclaredType::new(self.home_of(&name, from), name)
    }

    #[must_use]
    pub fn home_of(&self, name: &Name, from: Home<'file>) -> Home<'file> {
        let candidates = self.declared.get(name).map_or(&[][..], Vec::as_slice);
        candidates
            .iter()
            .find(|home| **home == from)
            .or_else(|| candidates.iter().find(|home| home.in_crate_with(from).is_some()))
            .or_else(|| candidates.first())
            .map_or(from, |home| *home)
    }

    fn reading(self, file: &'file SourceFile) -> TypeHomes<'file> {
        let mut homes = TypeHomes { reading: Home::containing(file), ..self };
        homes.visit_file(file.syntax());
        homes
    }

    fn declare(&mut self, name: Name) {
        let known = self.declared.entry(name).or_default();
        let fresh = known.iter().find(|home| **home == self.reading).map_or(Some(self.reading), |_| None);
        known.extend(fresh);
    }
}

impl<'ast> Visit<'ast> for TypeHomes<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || self.declare(Name::from(&item.ident)));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || self.declare(Name::from(&item.ident)));
    }
}

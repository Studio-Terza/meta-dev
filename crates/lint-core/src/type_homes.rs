use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{ItemEnum, ItemMod, ItemStruct, ItemUnion};

use lint_layout::Name;

use crate::declared_type::DeclaredType;
use crate::home::Home;
use crate::source_file::SourceFile;
use crate::walk::Walk;

#[derive(Debug, Default)]
pub struct TypeHomes {
    declared: BTreeMap<Name, Vec<DeclaredType>>,
    reading: Home,
}

impl TypeHomes {
    #[must_use]
    pub fn declared_in(files: &[&SourceFile]) -> TypeHomes {
        files.iter().fold(TypeHomes::default(), |homes, file| homes.reading(file))
    }

    #[must_use]
    pub fn resolve(&self, name: &Name, from: &Home) -> DeclaredType {
        let candidates = self.declared.get(name).map_or(&[][..], Vec::as_slice);
        candidates
            .iter()
            .find(|declared| declared.home() == from)
            .or_else(|| candidates.iter().find(|declared| declared.home().in_crate_with(from)))
            .or_else(|| candidates.first())
            .cloned()
            .unwrap_or_else(|| DeclaredType::new(from.clone(), name.clone()))
    }

    fn reading(self, file: &SourceFile) -> TypeHomes {
        let mut homes = TypeHomes { reading: Home::containing(file), ..self };
        homes.visit_file(file.syntax());
        homes
    }

    fn declare(&mut self, name: Name) {
        let declared = DeclaredType::new(self.reading.clone(), name.clone());
        let known = self.declared.entry(name).or_default();
        let fresh: Option<DeclaredType> = Some(declared).filter(|declared| !known.contains(declared));
        known.extend(fresh);
    }
}

impl<'ast> Visit<'ast> for TypeHomes {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || self.declare(Name::from(&item.ident)));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || self.declare(Name::from(&item.ident)));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        Walk::enter(&item.attrs, || self.declare(Name::from(&item.ident)));
    }
}

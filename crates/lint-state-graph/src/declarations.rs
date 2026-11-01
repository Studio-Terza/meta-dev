use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::visit::Visit;
use syn::{Fields, ItemEnum, ItemImpl, ItemMod, ItemType};

use lint_core::SourceFile;
use lint_core::TypeName;
use lint_core::Walk;
use lint_layout::Name;

#[derive(Default)]
pub struct Declarations {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    enum_homes: BTreeMap<Name, BTreeSet<PathBuf>>,
    sum_types: BTreeSet<Name>,
    path: PathBuf,
}

impl From<&[&SourceFile]> for Declarations {
    fn from(files: &[&SourceFile]) -> Declarations {
        let declarations = files.iter().fold(Declarations::default(), |mut declarations, file| {
            file.path().clone_into(&mut declarations.path);
            declarations.visit_file(file.syntax());
            declarations
        });
        let resolved: BTreeSet<TypeName> =
            declarations.vertices.iter().map(|vertex| declarations.resolve(vertex.clone())).collect();
        Declarations { vertices: resolved, ..declarations }
    }
}

impl Declarations {
    pub fn resolve(&self, name: TypeName) -> TypeName {
        self.aliases.get(&name).cloned().unwrap_or(name)
    }

    pub fn is_vertex(&self, name: &TypeName) -> bool {
        self.vertices.contains(name)
    }

    pub fn is_foreign_sum_type(&self, enum_name: &Name, path: &Path) -> bool {
        self.sum_types.contains(enum_name) && self.enum_homes.get(enum_name).is_some_and(|homes| !homes.contains(path))
    }

    fn vertex(item: &ItemImpl) -> Option<TypeName> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|segment| segment.ident == "State")
            .map(|_| TypeName::from_type(&item.self_ty))
    }

    fn record_enum(&mut self, item: &ItemEnum) {
        let name = Name::from(&item.ident);
        self.enum_homes.entry(name.clone()).or_default().insert(self.path.clone());
        self.sum_types.extend(Self::is_sum_type(item).then_some(name));
    }

    fn is_sum_type(item: &ItemEnum) -> bool {
        item.variants.iter().any(|variant| !matches!(variant.fields, Fields::Unit))
    }
}

impl<'ast> Visit<'ast> for Declarations {
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || {
            self.vertices.extend(Self::vertex(item));
            syn::visit::visit_item_impl(self, item);
        });
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.aliases.insert(TypeName::named(&item.ident), TypeName::from_type(&item.ty));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || self.record_enum(item));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }
}

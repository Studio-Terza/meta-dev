use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::visit::Visit;
use syn::{Fields, ItemEnum, ItemImpl, ItemType};

use lint_core::Name;
use lint_core::SourceFile;
use lint_core::TypeName;

#[derive(Default)]
pub struct Declarations {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    sum_types: BTreeMap<Name, BTreeSet<PathBuf>>,
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
        self.sum_types.get(enum_name).is_some_and(|homes| !homes.contains(path))
    }

    fn vertex(item: &ItemImpl) -> Option<TypeName> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|segment| segment.ident == "State")
            .map(|_| TypeName::from_type(&item.self_ty))
    }

    fn record_sum_type(&mut self, name: Option<Name>) -> bool {
        match name {
            Some(name) => self.sum_types.entry(name).or_default().insert(self.path.clone()),
            None => false,
        }
    }

    fn sum_type(item: &ItemEnum) -> Option<Name> {
        item.variants.iter().any(|variant| !matches!(variant.fields, Fields::Unit)).then(|| Name::from(&item.ident))
    }
}

impl<'ast> Visit<'ast> for Declarations {
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.vertices.extend(Self::vertex(item));
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.aliases.insert(TypeName::named(&item.ident), TypeName::from_type(&item.ty));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.record_sum_type(Self::sum_type(item));
    }
}

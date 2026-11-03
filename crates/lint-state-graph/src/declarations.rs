mod vertex;

use std::collections::{BTreeMap, BTreeSet};
use syn::visit::Visit;
use syn::{Fields, ItemEnum, ItemImpl, ItemMod, ItemType};

use lint_core::Home;
use lint_core::SourceFile;
use lint_core::TypeHomes;
use lint_core::TypeName;
use lint_core::Walk;
use lint_layout::Name;
pub use vertex::Vertex;

#[derive(Default)]
pub struct Declarations<'file> {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    homes: TypeHomes<'file>,
    sum_types: BTreeMap<Home<'file>, BTreeSet<Name>>,
    reading: BTreeSet<Name>,
}

impl<'file> From<&[&'file SourceFile]> for Declarations<'file> {
    fn from(files: &[&'file SourceFile]) -> Declarations<'file> {
        let seeded = Declarations { homes: TypeHomes::declared_in(files), ..Declarations::default() };
        files
            .iter()
            .fold(seeded, |mut declarations, file| {
                declarations.visit_file(file.syntax());
                declarations.filed(Home::containing(file))
            })
            .with_resolved_vertices()
    }
}

impl<'file> Declarations<'file> {
    pub fn resolved<'name>(&'name self, name: &'name TypeName) -> &'name TypeName {
        self.aliases.get(name).unwrap_or(name)
    }

    pub fn vertex_named(&self, name: &TypeName) -> Option<Vertex<'_>> {
        self.vertices.get(name).map(Vertex::new)
    }

    pub fn foreign_sum_type(&self, enum_name: &Name, from: &Home<'file>) -> Option<&Name> {
        let home = self.homes.home_of(enum_name, *from);
        self.sum_types.get(&home).and_then(|names| names.get(enum_name)).filter(|_| home != *from)
    }

    fn filed(mut self, home: Home<'file>) -> Declarations<'file> {
        let enums = std::mem::take(&mut self.reading);
        self.sum_types.entry(home).or_default().extend(enums);
        self
    }

    fn with_resolved_vertices(mut self) -> Declarations<'file> {
        let vertices = std::mem::take(&mut self.vertices);
        self.vertices =
            vertices.into_iter().map(|vertex| self.aliases.get(&vertex).cloned().unwrap_or(vertex)).collect();
        self
    }

    fn vertex(item: &ItemImpl) -> Option<TypeName> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|segment| segment.ident == "State")
            .map(|_| TypeName::from_type(&item.self_ty))
    }

    fn sum_type(item: &ItemEnum) -> Option<&ItemEnum> {
        item.variants
            .iter()
            .find(|variant| matches!(variant.fields, Fields::Named(_) | Fields::Unnamed(_)))
            .map(|_| item)
    }
}

impl<'ast> Visit<'ast> for Declarations<'_> {
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
        Walk::enter(&item.attrs, || self.reading.extend(Self::sum_type(item).map(|found| Name::from(&found.ident))));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }
}

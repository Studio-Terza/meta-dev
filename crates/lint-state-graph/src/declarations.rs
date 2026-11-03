use std::collections::{BTreeMap, BTreeSet};
use syn::visit::Visit;
use syn::{Fields, ItemEnum, ItemImpl, ItemMod, ItemType};

use lint_core::Home;
use lint_core::SourceFile;
use lint_core::TypeHomes;
use lint_core::TypeName;
use lint_core::Walk;
use lint_layout::Name;

#[derive(Default)]
pub struct Declarations {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    homes: TypeHomes,
    sum_types: BTreeMap<Home, BTreeSet<Name>>,
    reading: BTreeSet<Name>,
}

impl From<&[&SourceFile]> for Declarations {
    fn from(files: &[&SourceFile]) -> Declarations {
        let seeded = Declarations { homes: TypeHomes::declared_in(files), ..Declarations::default() };
        files
            .iter()
            .fold(seeded, |mut declarations, file| {
                declarations.visit_file(file.syntax());
                declarations.filed(Home::containing(file))
            })
            .resolved()
    }
}

impl Declarations {
    pub fn resolve(&self, name: TypeName) -> TypeName {
        self.aliases.get(&name).cloned().unwrap_or(name)
    }

    pub fn vertex_named(&self, name: &TypeName) -> Option<&TypeName> {
        self.vertices.get(name)
    }

    pub fn foreign_sum_type(&self, enum_name: &Name, from: &Home) -> Option<&Name> {
        let declared = self.homes.resolve(enum_name, from);
        let home = declared.home();
        self.sum_types.get(home).and_then(|names| names.get(enum_name)).filter(|_| home != from)
    }

    fn filed(mut self, home: Home) -> Declarations {
        let enums = std::mem::take(&mut self.reading);
        self.sum_types.entry(home).or_default().extend(enums);
        self
    }

    fn resolved(mut self) -> Declarations {
        let vertices = std::mem::take(&mut self.vertices);
        self.vertices = vertices.into_iter().map(|vertex| self.resolve(vertex)).collect();
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
        item.variants.iter().find(|variant| !matches!(variant.fields, Fields::Unit)).map(|_| item)
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
        Walk::enter(&item.attrs, || self.reading.extend(Self::sum_type(item).map(|found| Name::from(&found.ident))));
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }
}

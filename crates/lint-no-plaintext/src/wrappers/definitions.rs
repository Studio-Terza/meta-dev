mod conversions;
mod shape;

use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{Field, ItemImpl, ItemMod, ItemStruct};

use lint_core::{SourceFile, Syntax, Walk};
use lint_layout::Name;

use crate::attributes::Attributes;
use crate::wrappers::Wrappers;
use conversions::Conversions;
use shape::Shape;

#[derive(Default)]
pub struct Definitions<'file> {
    shapes: Vec<Shape<'file>>,
    names: BTreeSet<Name>,
    validated: BTreeSet<Name>,
}

impl<'file> Definitions<'file> {
    const DESERIALIZE: &'static str = "Deserialize";

    pub fn collected_from(files: &[&'file SourceFile]) -> Definitions<'file> {
        files.iter().fold(Definitions::default(), |mut definitions, file| {
            definitions.visit_file(file.syntax());
            definitions
        })
    }

    pub fn plain_among(&self, settled: &Wrappers) -> BTreeSet<Name> {
        self.names
            .difference(&self.validated)
            .filter(|name| {
                self.shapes
                    .iter()
                    .filter_map(|shape| shape.answers_to(name))
                    .all(|shape| shape.holds_text(settled).is_some())
            })
            .cloned()
            .collect()
    }

    fn only_field(item: &ItemStruct) -> Option<&Field> {
        Some(&item.fields).filter(|fields| fields.len() == 1).and_then(|fields| fields.iter().next())
    }

    fn validated_owner(item: &ItemImpl) -> Option<Name> {
        let deserializing = item
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|last| last.ident == Self::DESERIALIZE);
        let owner = Syntax::last_name(&item.self_ty);
        deserializing.and(Conversions::called_in(item)).and(owner)
    }
}

impl<'file> Visit<'file> for Definitions<'file> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'file ItemStruct) {
        Walk::enter(&item.attrs, || {
            self.shapes.push(Shape::new(
                &item.ident,
                Self::only_field(item)
                    .filter(|_| Attributes::flattened_from(&item.attrs).validation().is_none())
                    .map(|field| &field.ty),
            ));
            self.names.insert(Name::from(&item.ident));
        });
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        self.validated.extend(Self::validated_owner(item));
        syn::visit::visit_item_impl(self, item);
    }
}

mod conversions;
mod single_field;

use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{Field, ItemImpl, ItemMod, ItemStruct, Type};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;

use crate::attributes::Attributes;
use conversions::Conversions;
pub use single_field::SingleField;

#[derive(Default)]
pub struct Definitions {
    single_fields: Vec<SingleField>,
    validated: BTreeSet<Name>,
}

impl Definitions {
    const DESERIALIZE: &'static str = "Deserialize";

    pub fn collected_from(files: &[&SourceFile]) -> Definitions {
        files.iter().fold(Definitions::default(), |mut definitions, file| {
            definitions.visit_file(file.syntax());
            definitions
        })
    }

    pub fn single_fields(&self) -> impl Iterator<Item = &SingleField> {
        self.single_fields.iter().filter(|field| !self.validated.contains(field.name()))
    }

    fn only_field(item: &ItemStruct) -> Option<&Field> {
        let fields: Vec<&Field> = item.fields.iter().collect();
        match fields.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    fn validated_owner(item: &ItemImpl) -> Option<Name> {
        let deserializing = item
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|last| last.ident == Self::DESERIALIZE);
        let owner = match &*item.self_ty {
            Type::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        };
        owner.filter(|_| deserializing.is_some() && Conversions::called_in(item))
    }
}

impl<'ast> Visit<'ast> for Definitions {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || {
            self.single_fields.extend(
                Self::only_field(item)
                    .filter(|_| Attributes::flattened_from(&item.attrs).validation().is_none())
                    .map(|field| SingleField::new(Name::from(&item.ident), field.ty.clone())),
            );
        });
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.validated.extend(Self::validated_owner(item));
        syn::visit::visit_item_impl(self, item);
    }
}

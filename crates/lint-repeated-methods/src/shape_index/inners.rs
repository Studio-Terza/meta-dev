use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{Fields, ItemStruct, Type};

use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct Inners(BTreeMap<Name, Type>);

impl Inners {
    pub fn gathered_from(files: &[&SourceFile]) -> Inners {
        files.iter().fold(Inners::default(), |mut inners, file| {
            inners.visit_file(file.syntax());
            inners
        })
    }

    pub fn inner_of(&self, name: &Name) -> Option<&Type> {
        self.0.get(name)
    }
}

impl<'ast> Visit<'ast> for Inners {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let only = match &item.fields {
            Fields::Named(fields) if fields.named.len() == 1 => fields.named.first(),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields.unnamed.first(),
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => None,
        };
        self.0.extend(only.map(|field| (Name::from(&item.ident), field.ty.clone())));
    }
}

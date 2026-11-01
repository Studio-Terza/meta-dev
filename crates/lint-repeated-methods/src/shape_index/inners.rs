use std::collections::BTreeMap;

use syn::visit::Visit;
use syn::{Fields, ItemStruct, Type};

use lint_core::{DeclaredType, Home, SourceFile};
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct Inners {
    by_type: BTreeMap<DeclaredType, Type>,
    reading: Home,
}

impl Inners {
    pub fn gathered_from(files: &[&SourceFile]) -> Inners {
        files.iter().fold(Inners::default(), |inners, file| {
            let mut inners = Inners { reading: Home::containing(file), ..inners };
            inners.visit_file(file.syntax());
            inners
        })
    }

    pub fn inner_of(&self, owner: &DeclaredType) -> Option<&Type> {
        self.by_type.get(owner)
    }
}

impl<'ast> Visit<'ast> for Inners {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let only = match &item.fields {
            Fields::Named(fields) if fields.named.len() == 1 => fields.named.first(),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields.unnamed.first(),
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => None,
        };
        let owner = DeclaredType::new(self.reading.clone(), Name::from(&item.ident));
        self.by_type.extend(only.map(|field| (owner, field.ty.clone())));
    }
}

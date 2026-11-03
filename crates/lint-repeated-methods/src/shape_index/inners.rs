use std::collections::BTreeMap;
use std::mem;

use syn::visit::Visit;
use syn::{Fields, ItemStruct, Type};

use lint_core::{Home, SourceFile};
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct Inners<'file> {
    by_home: BTreeMap<Home<'file>, BTreeMap<Name, &'file Type>>,
    reading: BTreeMap<Name, &'file Type>,
}

impl<'file> Inners<'file> {
    pub fn gathered_from(files: &[&'file SourceFile]) -> Inners<'file> {
        files.iter().fold(Inners::default(), |mut inners, file| {
            inners.visit_file(file.syntax());
            inners.by_home.entry(Home::containing(file)).or_default().extend(mem::take(&mut inners.reading));
            inners
        })
    }

    pub fn inner_of(&self, home: &Home<'file>, name: &Name) -> Option<&'file Type> {
        self.by_home.get(home)?.get(name).copied()
    }
}

impl<'file> Visit<'file> for Inners<'file> {
    fn visit_item_struct(&mut self, item: &'file ItemStruct) {
        let only = match &item.fields {
            Fields::Named(fields) if fields.named.len() == 1 => fields.named.first(),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields.unnamed.first(),
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => None,
        };
        self.reading.extend(only.map(|field| (Name::from(&item.ident), &field.ty)));
    }
}

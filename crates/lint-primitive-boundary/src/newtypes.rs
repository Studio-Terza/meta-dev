use syn::visit::Visit;
use syn::{Fields, ItemStruct};

use crate::text_sites::Owner;
use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct Newtypes(Vec<Name>);

impl Newtypes {
    pub fn in_files(files: &[SourceFile]) -> Newtypes {
        let mut newtypes = Newtypes::default();
        files.iter().filter_map(SourceFile::shipped).for_each(|file| newtypes.visit_file(file.syntax()));
        newtypes
    }

    pub fn holds(&self, owner: &Owner) -> Option<&Name> {
        owner.name().and_then(|name| self.0.iter().find(|newtype| *newtype == name))
    }
}

impl<'ast> Visit<'ast> for Newtypes {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        match &item.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => self.0.push(Name::from(&item.ident)),
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => {}
        }
    }
}

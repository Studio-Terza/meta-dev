use syn::visit::Visit;
use syn::{ImplItemFn, ItemStruct};

use crate::owner::Owner;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug, Default)]
pub struct Newtypes(Vec<Name>);

impl Newtypes {
    const EDGES: [Word<'static>; 3] = words!["new", "try_new", "into_inner"];

    pub fn edge(&self, owner: &Owner, item: &ImplItemFn) -> Option<Name> {
        let name = Name::from(&item.sig.ident);
        self.holds(owner).and(name.matches::<Whole>(&Self::EDGES)).filter(|_| item.sig.inputs.len() == 1).map(|_| name)
    }

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
        self.0.extend(Syntax::newtype_field(&item.fields).map(|_| Name::from(&item.ident)));
    }
}

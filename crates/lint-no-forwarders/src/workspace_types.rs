use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{ItemEnum, ItemStruct, ItemTrait};

use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct WorkspaceTypes(BTreeSet<Name>);

impl From<&SourceFile> for WorkspaceTypes {
    fn from(file: &SourceFile) -> WorkspaceTypes {
        let mut types = WorkspaceTypes::default();
        types.visit_file(file.syntax());
        types
    }
}

impl FromIterator<WorkspaceTypes> for WorkspaceTypes {
    fn from_iter<Parts: IntoIterator<Item = WorkspaceTypes>>(parts: Parts) -> WorkspaceTypes {
        WorkspaceTypes(parts.into_iter().flat_map(|part| part.0).collect())
    }
}

impl WorkspaceTypes {
    pub fn declares(&self, name: &Name) -> Option<&Name> {
        self.0.get(name)
    }
}

impl<'ast> Visit<'ast> for WorkspaceTypes {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.0.insert(Name::from(&item.ident));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.0.insert(Name::from(&item.ident));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.0.insert(Name::from(&item.ident));
    }
}

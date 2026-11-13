use std::collections::BTreeSet;

use syn::{ItemEnum, ItemStruct, ItemTrait};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};

#[derive(Debug, Default)]
pub struct WorkspaceTypes(BTreeSet<Name>);

#[derive(Debug)]
pub struct TypeGathering {
    types: WorkspaceTypes,
    kinds: &'static [Kind],
}

impl From<&SourceFile> for TypeGathering {
    fn from(file: &SourceFile) -> TypeGathering {
        let kinds = file.shipped().map_or(&[][..], |_| &[Kind::ItemStruct, Kind::ItemEnum, Kind::ItemTrait][..]);
        TypeGathering { types: WorkspaceTypes::default(), kinds }
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

impl TypeGathering {
    fn declared(&mut self, name: &syn::Ident) -> Descend {
        self.types.0.insert(Name::from(name));
        Descend::Over
    }
}

impl<'ast> Watch<'ast> for TypeGathering {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        self.declared(&item.ident)
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        self.declared(&item.ident)
    }

    fn on_item_trait(&mut self, item: &'ast ItemTrait) -> Descend {
        self.declared(&item.ident)
    }
}

impl Gathers<WorkspaceTypes> for TypeGathering {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> WorkspaceTypes {
        self.types
    }
}

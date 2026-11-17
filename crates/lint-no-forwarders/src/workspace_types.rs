use std::collections::BTreeSet;

use syntax::{ItemEnum, ItemStruct, ItemTrait};

use lint_core::Extended;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

#[derive(Debug, Default)]
pub struct WorkspaceTypes(BTreeSet<Name>);

#[derive(Debug)]
pub struct TypeGathering {
    kinds: &'static [Kind],
}

impl From<&SourceFile> for TypeGathering {
    fn from(file: &SourceFile) -> TypeGathering {
        let kinds = file.shipped().map_or(&[][..], |_| &[Kind::ItemStruct, Kind::ItemEnum, Kind::ItemTrait][..]);
        TypeGathering { kinds }
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

impl WorkspaceTypes {
    fn declared(self, name: &syntax::Ident) -> Visit<WorkspaceTypes> {
        Visit::new(WorkspaceTypes(self.0.extended([Name::from(name)])), Descend::Over)
    }
}

impl Fold for TypeGathering {
    type Found = WorkspaceTypes;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_struct(
        &self,
        found: WorkspaceTypes,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
    ) -> Visit<WorkspaceTypes> {
        found.declared(&item.ident)
    }

    fn on_item_enum(&self, found: WorkspaceTypes, item: &ItemEnum, _: &Facts<'_, ItemEnum>) -> Visit<WorkspaceTypes> {
        found.declared(&item.ident)
    }

    fn on_item_trait(
        &self,
        found: WorkspaceTypes,
        item: &ItemTrait,
        _: &Facts<'_, ItemTrait>,
    ) -> Visit<WorkspaceTypes> {
        found.declared(&item.ident)
    }
}

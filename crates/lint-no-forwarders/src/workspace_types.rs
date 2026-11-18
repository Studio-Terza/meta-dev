use std::collections::BTreeSet;

use algebra::{Monoid, Semigroup};
use syntax::{ItemEnum, ItemStruct, ItemTrait};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Sweep, Watching};

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
        Monoid::concat(parts)
    }
}

impl Semigroup for WorkspaceTypes {
    fn combine(mut self, other: WorkspaceTypes) -> WorkspaceTypes {
        self.0.extend(other.0);
        self
    }
}

impl Monoid for WorkspaceTypes {}

impl WorkspaceTypes {
    pub fn declares(&self, name: &Name) -> Option<&Name> {
        self.0.get(name)
    }
}

impl TypeGathering {
    fn declared(name: &syntax::Ident) -> Contributed<(), WorkspaceTypes> {
        Contributed::new((), WorkspaceTypes(BTreeSet::from([Name::from(name)])), Descend::Over)
    }
}

impl Contribute for TypeGathering {
    type Open = ();
    type Part = WorkspaceTypes;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_struct(&self, item: &ItemStruct, _: &Facts<'_, ItemStruct>) -> Contributed<(), WorkspaceTypes> {
        Self::declared(&item.ident)
    }

    fn on_item_enum(&self, item: &ItemEnum, _: &Facts<'_, ItemEnum>) -> Contributed<(), WorkspaceTypes> {
        Self::declared(&item.ident)
    }

    fn on_item_trait(&self, item: &ItemTrait, _: &Facts<'_, ItemTrait>) -> Contributed<(), WorkspaceTypes> {
        Self::declared(&item.ident)
    }
}

impl Gathers<WorkspaceTypes> for Contributing<TypeGathering> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> WorkspaceTypes {
        self.concluded(|_, (), types| types)
    }
}

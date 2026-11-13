use code_model::{
    Function, Implementation, ItemDeclaration, ItemId, ItemKind, Model, Owner, Parameter, Placed, Reach, Region,
    Resolved, Seat,
};
use lint_layout::Name;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Stocked<'model> {
    placed: Placed<'model, &'model ItemDeclaration>,
    name: &'model Name,
    kind: ItemKind,
}

#[derive(Debug)]
pub(crate) struct Catalog<'model> {
    declared: Vec<Stocked<'model>>,
    functions: Vec<Placed<'model, &'model Function>>,
    implementations: Vec<Placed<'model, &'model Implementation>>,
}

impl<'model> Stocked<'model> {
    pub(crate) fn placed(self) -> Placed<'model, &'model ItemDeclaration> {
        self.placed
    }

    pub(crate) fn name(self) -> &'model Name {
        self.name
    }
}

impl<'model> From<&'model Model> for Catalog<'model> {
    fn from(model: &'model Model) -> Catalog<'model> {
        Catalog {
            declared: model
                .items()
                .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
                .filter_map(|placed| {
                    let item = model.declared(placed.entity().declared())?;
                    Some(Stocked { placed, name: item.as_ref(), kind: item.kind() })
                })
                .collect(),
            functions: model
                .functions()
                .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
                .filter(|placed| placed.entity().called().is_some())
                .collect(),
            implementations: model
                .implementations()
                .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
                .filter(|placed| placed.entity().written().is_some())
                .collect(),
        }
    }
}

impl<'model> Catalog<'model> {
    pub(crate) const TYPES: [ItemKind; 2] = [ItemKind::Struct, ItemKind::Enum];
    pub(crate) const OWNERS: [Owner; 3] = [Owner::TraitDecl, Owner::Inherent, Owner::TraitImpl];

    pub(crate) fn declared(&self, kinds: &[ItemKind]) -> impl Iterator<Item = Stocked<'model>> {
        let wanted = kinds.to_vec();
        self.declared.clone().into_iter().filter(move |stocked| wanted.contains(&stocked.kind))
    }

    pub(crate) fn generics(&self) -> impl Iterator<Item = Placed<'model, &'model Implementation>> + '_ {
        self.implementations
            .iter()
            .copied()
            .filter_map(|placed| AsRef::<[Parameter]>::as_ref(placed.entity()).first().map(|_| placed))
    }

    pub(crate) fn owned_by(&self, id: ItemId, owner: Owner) -> impl Iterator<Item = &'model Function> + '_ {
        self.functions
            .iter()
            .map(|placed| placed.entity())
            .filter(move |function| function.owner() == owner)
            .filter(move |function| AsRef::<Seat>::as_ref(*function).owner() == Some(id))
    }

    pub(crate) fn seated(
        &self,
        block: Placed<'model, &'model Implementation>,
    ) -> impl Iterator<Item = &'model Function> + '_ {
        let implementation = block.entity();
        let implemented = implementation.interface_name();
        self.functions
            .iter()
            .filter(move |placed| placed.site().path() == block.site().path())
            .map(|placed| placed.entity())
            .filter(move |function| AsRef::<Seat>::as_ref(*function).subject() == Some(implementation.subject_type()))
            .filter(move |function| AsRef::<Seat>::as_ref(*function).implemented() == implemented)
    }

    pub(crate) fn implementers(&self, id: ItemId) -> impl Iterator<Item = &'model Name> + '_ {
        self.implementations
            .iter()
            .map(|placed| placed.entity())
            .filter(move |implementation| implementation.implemented().and_then(Resolved::crate_item) == Some(id))
            .map(Implementation::subject)
    }

    pub(crate) fn implemented_by(&self, id: ItemId) -> impl Iterator<Item = &'model Implementation> + '_ {
        self.implementations.iter().map(|placed| placed.entity()).filter(move |implementation| {
            implementation.subject_type().named().map(AsRef::<Resolved>::as_ref).and_then(Resolved::crate_item)
                == Some(id)
        })
    }
}

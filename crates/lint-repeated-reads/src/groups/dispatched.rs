use std::collections::BTreeSet;

use graph::{Dispatch, Function, ItemId, ItemKind, ItemPath, Model, Owner, Reader, Resolved, Seat, TypeRef};
use lint_layout::Name;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Method<'model> {
    owner: ItemId,
    name: &'model Name,
}

impl<'model> Method<'model> {
    fn of_function(function: &'model Function) -> Option<Method<'model>> {
        let seat: &Seat = function.as_ref();
        let subject = seat.subject().and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref);
        let owner = seat.owner().or_else(|| subject?.crate_item())?;
        Some(Method { owner, name: function.as_ref() })
    }
}

#[derive(Debug)]
pub struct Dispatched<'model> {
    model: &'model Model,
    objects: BTreeSet<&'model Resolved>,
    traits: BTreeSet<ItemId>,
    associated: BTreeSet<Method<'model>>,
}

impl<'model> Dispatched<'model> {
    pub fn in_model(model: &'model Model) -> Dispatched<'model> {
        let functions = || model.functions().map(graph::Placed::entity);
        let declared: BTreeSet<Method<'model>> = functions()
            .filter(|function| function.owner() == Owner::TraitDecl)
            .filter_map(Method::of_function)
            .collect();
        let associated = functions().filter_map(Method::of_function).collect();
        let empty = Dispatched { model, objects: BTreeSet::new(), traits: BTreeSet::new(), associated };
        let uses = model.dispatches().iter();
        uses.fold(empty, |mut found, used| {
            let member = AsRef::<Resolved>::as_ref(used).member().map(graph::Member::owner);
            found.traits.extend(member.filter(|owner| Self::method_of(&declared, *owner, used).is_some()));
            found.objects.extend(member.map_or(Some(AsRef::<Resolved>::as_ref(used)), |_| None));
            found
        })
    }

    pub fn covers(&self, implemented: &Resolved) -> Option<()> {
        let by_item = implemented.crate_item().and_then(|item| self.traits.get(&item)).map(|_| ());
        by_item.or_else(|| self.objects.get(implemented).map(|_| ()))
    }

    pub fn constructs(&self, reader: &Reader) -> Option<()> {
        let owner = reader.resolved();
        let built = reader.constructing().map(|_| ());
        let generic = reader.member().and(owner.parameter()).map(|_| ());
        let variant = reader.member().zip(owner.crate_item()).and_then(|(name, item)| self.variant(item, name));
        built.or(generic).or(variant)
    }

    fn variant(&self, owner: ItemId, name: &Name) -> Option<()> {
        let declared = self.model.declared(owner).filter(|item| item.kind() == ItemKind::Enum)?;
        let method = self.associated.get(&Method { owner, name });
        method.map_or(Some(declared), |_| None).map(|_| ())
    }

    fn method_of(declared: &BTreeSet<Method<'model>>, owner: ItemId, used: &Dispatch) -> Option<()> {
        AsRef::<ItemPath>::as_ref(used).last().and_then(|name| declared.get(&Method { owner, name })).map(|_| ())
    }
}

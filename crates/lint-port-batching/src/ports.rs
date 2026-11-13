use std::collections::{BTreeMap, BTreeSet};

use code_model::{
    Bounds, Callable, Element, Ground, Grouped, Held, Holding, ItemId, ItemKind, ItemPath, Model, Named, Outline,
    Owner, Parameter, Placed, Projection, Region, Resolved, Seat, Sited, TypeRef,
};
use lint_layout::{Name, Whole, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ported {
    Port,
    Other,
}

#[derive(Debug)]
pub(crate) struct Ports<'model> {
    traits: BTreeSet<ItemId>,
    paced: BTreeSet<&'model Name>,
    holders: BTreeMap<ItemId, BTreeSet<&'model Name>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Typing<'ports, 'model> {
    ports: &'ports Ports<'model>,
    generics: &'ports [&'model Parameter],
}

#[derive(Clone, Copy, Debug)]
struct Holder<'model> {
    owner: ItemId,
    field: &'model Name,
}

impl<'model> From<&'model Model> for Ports<'model> {
    fn from(model: &'model Model) -> Ports<'model> {
        let traits: BTreeSet<ItemId> = model
            .items()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).ground() == Ground::Ports)
            .map(|placed| placed.entity().declared())
            .filter(|id| model.declared(*id).is_some_and(|item| item.kind() == ItemKind::Trait))
            .collect();
        let paced = model
            .functions()
            .map(Placed::entity)
            .filter(|function| function.owner() == Owner::TraitDecl)
            .filter(|function| AsRef::<Seat>::as_ref(*function).owner().is_some_and(|owner| traits.contains(&owner)))
            .filter(|function| {
                let spelled = function.signature().written().filter_map(TypeRef::named).filter_map(Named::last_spelled);
                spelled.into_iter().any(|name| name.matches::<Whole>(&Self::PACING).is_some())
            })
            .map(AsRef::<Name>::as_ref)
            .collect();
        let bare = Ports { traits, paced, holders: BTreeMap::new() };
        let holders = bare.held(model);
        Ports { holders, ..bare }
    }
}

impl<'model> Ports<'model> {
    const PACING: [Word<'static>; 1] = words!["Pacing"];

    pub(crate) fn paces(&self, method: &Name) -> Option<&&'model Name> {
        self.paced.get(method)
    }

    pub(crate) fn fields(&self, owner: ItemId) -> Option<&BTreeSet<&'model Name>> {
        self.holders.get(&owner)
    }

    pub(crate) fn typing<'ports>(&'ports self, generics: &'ports [&'model Parameter]) -> Typing<'ports, 'model> {
        Typing { ports: self, generics }
    }

    fn held(&self, model: &'model Model) -> BTreeMap<ItemId, BTreeSet<&'model Name>> {
        let typing = Typing { ports: self, generics: &[] };
        let bounded: BTreeSet<ItemId> = model
            .holdings()
            .map(Placed::entity)
            .filter(|holding| holding.held() == Held::StructParameter)
            .filter(|holding| AsRef::<TypeRef>::as_ref(*holding).outlined(typing) == Ported::Port)
            .filter_map(Holding::declares)
            .collect();
        model
            .holdings()
            .map(Placed::entity)
            .filter(|holding| holding.held() == Held::StructField)
            .filter_map(|holding| Holder::ported(holding, typing, &bounded))
            .fold(BTreeMap::new(), |mut holders: BTreeMap<ItemId, BTreeSet<&Name>>, holder| {
                holders.entry(holder.owner).or_default().insert(holder.field);
                holders
            })
    }
}

impl<'model> Holder<'model> {
    fn ported(
        holding: &'model Holding,
        typing: Typing<'_, 'model>,
        bounded: &BTreeSet<ItemId>,
    ) -> Option<Holder<'model>> {
        let owner = holding.declares()?;
        let ty: &TypeRef = holding.as_ref();
        let named = Some(ty.outlined(typing)).filter(|ported| *ported == Ported::Port);
        let generic = ty
            .dereferenced()
            .named()
            .map(AsRef::<Resolved>::as_ref)
            .and_then(Resolved::parameter)
            .filter(|_| bounded.contains(&owner))
            .map(|_| Ported::Port);
        named.or(generic).map(|_| Holder { owner, field: holding.as_ref() })
    }
}

impl Typing<'_, '_> {
    const POINTERS: [Word<'static>; 3] = words!["Box", "Rc", "Arc"];

    fn declared(self, id: ItemId) -> Ported {
        let held = self.ports.holders.get(&id).map(|_| Ported::Port);
        let traited = self.ports.traits.get(&id).map(|_| Ported::Port);
        traited.or(held).unwrap_or(Ported::Other)
    }

    fn generic(self, name: &Name) -> Option<Ported> {
        let parameter = self.generics.iter().rev().find(|parameter| AsRef::<Name>::as_ref(**parameter) == name)?;
        parameter.bounds().iter().map(|bound| bound.outlined(self)).find(|ported| *ported == Ported::Port)
    }
}

impl Outline for Typing<'_, '_> {
    type Kind = Ported;

    fn named(self, named: &Named<Sited>) -> Ported {
        let resolved: &Resolved = named.as_ref();
        let pointer = resolved
            .std()
            .and_then(ItemPath::last)
            .filter(|last| last.matches::<Whole>(&Self::POINTERS).is_some())
            .and_then(|_| named.arguments().first())
            .map(|inner| inner.outlined(self));
        let generic = || resolved.parameter().and_then(|name| self.generic(name));
        let declared = || resolved.crate_item().map(|id| self.declared(id));
        pointer.or_else(generic).or_else(declared).unwrap_or(Ported::Other)
    }

    fn grouped(self, _parts: Grouped<'_>) -> Ported {
        Ported::Other
    }

    fn sequence(self, _element: Element<'_>) -> Ported {
        Ported::Other
    }

    fn bounded(self, bounds: Bounds<'_>) -> Ported {
        bounds
            .as_ref()
            .iter()
            .map(|bound| bound.outlined(self))
            .find(|ported| *ported == Ported::Port)
            .unwrap_or(Ported::Other)
    }

    fn projected(self, _projection: &Projection<Sited>) -> Ported {
        Ported::Other
    }

    fn callable(self, _callable: &Callable<Sited>) -> Ported {
        Ported::Other
    }

    fn other(self) -> Ported {
        Ported::Other
    }
}

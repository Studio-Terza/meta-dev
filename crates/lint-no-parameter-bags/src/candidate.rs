mod finding;

use std::collections::BTreeSet;
use std::path::Path;

use body_lifts::Body;
use graph::{
    Consumers, Domain, Function, Implementation, ItemDeclaration, ItemId, ItemKind, ItemPath, Model, Origin, Owner,
    Placed, Reach, Region, Resolved, Seat,
};
use lint_core::Violation;
use lint_layout::Name;

use finding::Finding;

#[derive(Debug)]
pub struct Carriers(BTreeSet<ItemId>);

impl From<&Model> for Carriers {
    fn from(model: &Model) -> Carriers {
        let traited = model.implementations().filter_map(|placed| Carriers::traited(placed.entity())?.subject_item());
        Carriers(traited.chain(model.functions().filter_map(|placed| Method::carried(placed.entity()))).collect())
    }
}

impl Carriers {
    fn traited(implementation: &Implementation) -> Option<&Implementation> {
        Some(implementation.origin()).filter(|origin| *origin == Origin::Written)?;
        implementation.interface().map(|_| implementation)
    }

    fn carries(&self, item: ItemId) -> Option<ItemId> {
        self.0.get(&item).copied()
    }
}

#[derive(Debug)]
struct Method;

impl Method {
    fn carried(function: &Function) -> Option<ItemId> {
        let inherent = Some(function.owner()).filter(|owner| *owner == Owner::Inherent);
        let owner = inherent.and(AsRef::<Seat>::as_ref(function).owner())?;
        let shape = function.definition()?.shape();
        Method::accessor(function, shape)
            .or_else(|| Method::constructor(function, shape, owner))
            .map_or(Some(owner), |_| None)
    }

    fn accessor(function: &Function, shape: Body) -> Option<Body> {
        let focused = Some(shape).filter(|body| *body == Body::Focused);
        let inputs = function.signature().inputs();
        focused.filter(|_| inputs.iter().all(|input| input.receiver().is_some()))
    }

    fn constructor(function: &Function, shape: Body, owner: ItemId) -> Option<Body> {
        let built = Some(shape).filter(|body| *body == Body::Built);
        let output = function.signature().output().written().and_then(|ty| ty.named());
        let own = output.map(AsRef::<Resolved>::as_ref).and_then(Resolved::crate_item).filter(|item| *item == owner);
        built.and(own).map(|_| Body::Built)
    }
}

#[derive(Debug)]
pub struct Bag<'model> {
    item: ItemId,
    name: &'model Name,
    file: &'model Path,
    root: &'model Path,
    placed: Placed<'model, &'model ItemDeclaration>,
}

impl<'model> Bag<'model> {
    pub fn found(model: &'model Model, placed: Placed<'model, &'model ItemDeclaration>) -> Option<Bag<'model>> {
        let item = placed.entity().declared();
        let declared = model.declared(item).filter(|found| found.kind() == ItemKind::Struct)?;
        let shipped = Some(AsRef::<Region>::as_ref(placed.entity()).reach()).filter(|reach| *reach == Reach::Shipped);
        let name: &Name = declared.as_ref();
        let path: ItemPath = declared.krate().into_iter().chain([name]).cloned().collect();
        let exported = model.exported(Domain::Types, &path).and_then(Resolved::crate_item).filter(|id| *id == item);
        let hidden = exported.map_or(Some(()), |_| None);
        shipped.and(hidden).map(|()| Bag {
            item,
            name,
            file: <&Path>::from(placed.site()),
            root: placed.root(),
            placed,
        })
    }

    pub fn bagged(self, carriers: &Carriers, consumers: &Consumers<'model>) -> Option<Violation<'model>> {
        let idle = carriers.carries(self.item).map_or(Some(()), |_| None);
        let files: BTreeSet<&Path> = consumers
            .files(self.item, self.root)
            .into_iter()
            .flatten()
            .copied()
            .filter(|file| *file != self.file)
            .collect();
        let sole = Some(files.len()).filter(|count| *count == 1);
        idle.and(sole).map(|_| Violation::new(self.placed.site(), Finding::ParameterBag { name: self.name }))
    }
}

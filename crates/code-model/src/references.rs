use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::boundary::ItemId;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::member::Member;
use crate::model::Model;
use crate::path_use::PathUse;
use crate::placed::Placed;
use crate::region::{Reach, Region, Seat};
use crate::resolved::Resolved;
use crate::sited::Sited;
use crate::type_ref::{Named, TypeRef, TypeUses};

#[derive(Debug, Default)]
struct Reached(BTreeSet<ItemId>);

impl TypeUses for Reached {
    fn named(&mut self, named: &Named<Sited>) {
        self.0.extend(AsRef::<Resolved>::as_ref(named).crate_item());
        named.each_inner_use(self);
    }
}

impl From<&Resolved> for Reached {
    fn from(resolved: &Resolved) -> Reached {
        Reached(resolved.crate_item().or_else(|| resolved.member().map(Member::owner)).into_iter().collect())
    }
}

impl<'ty> FromIterator<&'ty TypeRef> for Reached {
    fn from_iter<Types: IntoIterator<Item = &'ty TypeRef>>(types: Types) -> Reached {
        types.into_iter().fold(Reached::default(), |mut reached, ty| {
            ty.each_use(&mut reached);
            reached
        })
    }
}

#[derive(Debug)]
struct Owned {
    owner: Option<ItemId>,
    reached: Reached,
}

impl Owned {
    fn shipped(region: &impl AsRef<Region>, owner: Option<ItemId>, reached: Reached) -> Owned {
        let shipped = Some(region.as_ref().reach()).filter(|reach| *reach == Reach::Shipped);
        Owned { owner: shipped.and(owner), reached }
    }
}

impl From<&Holding> for Owned {
    fn from(holding: &Holding) -> Owned {
        Owned::shipped(holding, holding.declares(), [AsRef::<TypeRef>::as_ref(holding)].into_iter().collect())
    }
}

impl From<&Implementation> for Owned {
    fn from(implementation: &Implementation) -> Owned {
        let subject = implementation.subject_type().named().map(AsRef::<Resolved>::as_ref);
        let reached = implementation.implemented().map_or_default(Reached::from);
        Owned::shipped(implementation, subject.and_then(Resolved::crate_item), reached)
    }
}

impl From<&Function> for Owned {
    fn from(function: &Function) -> Owned {
        let owner = AsRef::<Seat>::as_ref(function).owner();
        Owned::shipped(function, owner, function.signature().written().collect())
    }
}

impl From<&PathUse> for Owned {
    fn from(used: &PathUse) -> Owned {
        Owned::shipped(used, AsRef::<Seat>::as_ref(used).owner(), Reached::from(used.resolved()))
    }
}

#[derive(Debug, Default)]
pub struct References<'model> {
    targets: BTreeMap<ItemId, BTreeSet<ItemId>>,
    roots: BTreeMap<ItemId, &'model Path>,
}

impl<'model> References<'model> {
    #[must_use]
    pub fn targets(&self) -> &BTreeMap<ItemId, BTreeSet<ItemId>> {
        &self.targets
    }

    #[must_use]
    pub fn root(&self, item: ItemId) -> Option<&'model Path> {
        self.roots.get(&item).copied()
    }
}

impl<'model> From<&'model Model> for References<'model> {
    fn from(model: &'model Model) -> References<'model> {
        model
            .holdings()
            .map(References::from)
            .chain(model.implementations().map(References::from))
            .chain(model.functions().map(References::from))
            .chain(model.mentions().map(References::from))
            .collect()
    }
}

impl<'model, Entity> From<Placed<'model, Entity>> for References<'model>
where
    Entity: Copy,
    Owned: From<Entity>,
{
    fn from(placed: Placed<'model, Entity>) -> References<'model> {
        let Owned { owner, reached } = Owned::from(placed.entity());
        owner.map_or_default(|found| References {
            targets: BTreeMap::from([(found, reached.0)]),
            roots: BTreeMap::from([(found, placed.root())]),
        })
    }
}

impl<'model> FromIterator<References<'model>> for References<'model> {
    fn from_iter<Parts: IntoIterator<Item = References<'model>>>(parts: Parts) -> References<'model> {
        parts.into_iter().fold(References::default(), |mut references, part| {
            part.targets
                .into_iter()
                .for_each(|(owner, targets)| references.targets.entry(owner).or_default().extend(targets));
            references.roots.extend(part.roots);
            references
        })
    }
}

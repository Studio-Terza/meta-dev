use std::collections::{BTreeMap, BTreeSet};

use code_model::{
    Function, Holding, Implementation, ItemId, Member, Model, Named, PathUse, Placed, Reach, Region, Resolved, Seat,
    Sited, TypeRef, TypeUses,
};
use lint_core::{Bound, Site, Violation};
use lint_item_graph::ItemGraph;
use lint_layout::Name;

mod tangle;

use tangle::Tangle;

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
pub(crate) struct Links<'model> {
    targets: BTreeMap<ItemId, BTreeSet<ItemId>>,
    sites: BTreeMap<ItemId, Site<'model>>,
}

impl<'model> Links<'model> {
    const TANGLE_ITEMS: Bound = Bound::Most(40);

    pub(crate) fn violations(self, model: &'model Model) -> Vec<Violation<'model>> {
        let sites = self.sites;
        ItemGraph::of_references(self.targets)
            .tangles()
            .into_iter()
            .filter_map(|members| {
                let site = members.iter().find_map(|member| sites.get(*member)).copied()?;
                let mut names: Vec<Name> = members
                    .into_iter()
                    .filter_map(|member| model.declared(*member))
                    .map(|item| AsRef::<Name>::as_ref(item).clone())
                    .collect();
                names.sort();
                let tangle = Tangle::new(names);
                Self::TANGLE_ITEMS.allows(tangle.items()).map_or(Some(Violation::new(site, tangle)), |_| None)
            })
            .collect()
    }
}

impl<'model, Entity> From<Placed<'model, Entity>> for Links<'model>
where
    Entity: Copy,
    Owned: From<Entity>,
{
    fn from(placed: Placed<'model, Entity>) -> Links<'model> {
        let Owned { owner, reached } = Owned::from(placed.entity());
        owner.map_or_default(|found| Links {
            targets: BTreeMap::from([(found, reached.0)]),
            sites: BTreeMap::from([(found, Site::of_file(placed.root()))]),
        })
    }
}

impl<'model> FromIterator<Links<'model>> for Links<'model> {
    fn from_iter<Parts: IntoIterator<Item = Links<'model>>>(parts: Parts) -> Links<'model> {
        parts.into_iter().fold(Links::default(), |mut links, part| {
            part.targets
                .into_iter()
                .for_each(|(owner, targets)| links.targets.entry(owner).or_default().extend(targets));
            links.sites.extend(part.sites);
            links
        })
    }
}

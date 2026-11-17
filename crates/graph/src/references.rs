use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use algebra::{Monoid, Semigroup};
use newtype::Arbitrary;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::{btree_map, btree_set};
use newtype::proptest::strategy::Strategy;

use crate::boundary::ItemId;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item::Item;
use crate::item_declaration::ItemDeclaration;
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
pub(crate) struct Owned {
    owner: Option<ItemId>,
    reached: Reached,
    shipped: Option<Reach>,
    traited: Option<ItemId>,
}

impl Owned {
    fn shipped(region: &impl AsRef<Region>, owner: Option<ItemId>, reached: Reached) -> Owned {
        let shipped = Some(region.as_ref().reach()).filter(|reach| *reach == Reach::Shipped);
        Owned { owner: shipped.and(owner), reached, shipped, traited: None }
    }

    fn traited(self, implemented: Option<&Resolved>) -> Owned {
        let traited = self.shipped.and(implemented).and_then(Resolved::crate_item);
        Owned { traited, ..self }
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
            .traited(implementation.implemented())
    }
}

impl From<&Function> for Owned {
    fn from(function: &Function) -> Owned {
        let seat: &Seat = function.as_ref();
        Owned::shipped(function, seat.owner(), function.signature().written().collect())
            .traited(seat.implemented_trait())
    }
}

impl From<&PathUse> for Owned {
    fn from(used: &PathUse) -> Owned {
        let seat: &Seat = used.as_ref();
        Owned::shipped(used, seat.owner(), Reached::from(used.resolved())).traited(seat.implemented_trait())
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(generate = References::sampled)]
pub struct References {
    targets: BTreeMap<ItemId, BTreeSet<ItemId>>,
    roots: BTreeMap<ItemId, PathBuf>,
}

impl References {
    const SAMPLED: Range<usize> = 0..4;

    #[must_use]
    pub fn targets(&self) -> &BTreeMap<ItemId, BTreeSet<ItemId>> {
        &self.targets
    }

    #[must_use]
    pub fn root(&self, item: ItemId) -> Option<&Path> {
        self.roots.get(&item).map(PathBuf::as_path)
    }

    fn sampled() -> impl Strategy<Value = References> {
        let drawn_targets = btree_map(any::<ItemId>(), btree_set(any::<ItemId>(), Self::SAMPLED), Self::SAMPLED);
        let drawn_roots = btree_map(any::<ItemId>(), any::<String>(), Self::SAMPLED);
        Strategy::prop_map((drawn_targets, drawn_roots), |(targets, roots)| References {
            targets,
            roots: roots.into_iter().map(|(item, root)| (item, PathBuf::from(root))).collect(),
        })
    }
}

impl References {
    pub(crate) fn homed<Entity>(model: &Model, placed: Placed<'_, Entity>) -> References
    where
        Entity: Copy,
        Owned: From<Entity>,
    {
        let Owned { owner, reached, traited, .. } = Owned::from(placed.entity());
        let at_home =
            |item: &ItemId| model.declared(*item).and_then(Item::krate).is_none_or(|krate| krate == placed.krate());
        let home = owner.filter(at_home);
        let foreign = home.map_or(owner, |_| None);
        let link = home.or_else(|| traited.filter(at_home));
        let targets: BTreeSet<ItemId> = reached.0.into_iter().chain(foreign).collect();
        link.map_or_else(References::identity, |found| References {
            targets: BTreeMap::from([(found, targets.into_iter().filter(|target| *target != found).collect())]),
            roots: BTreeMap::from([(found, placed.root().to_path_buf())]),
        })
    }
}

impl Semigroup for References {
    fn combine(mut self, part: References) -> References {
        part.targets.into_iter().for_each(|(owner, targets)| self.targets.entry(owner).or_default().extend(targets));
        self.roots.extend(part.roots);
        self
    }
}

impl Monoid for References {
    fn identity() -> References {
        References { targets: BTreeMap::new(), roots: BTreeMap::new() }
    }
}

type CrateFiles<'model> = BTreeMap<&'model Path, BTreeSet<&'model Path>>;

#[derive(Debug, Default)]
pub struct Consumers<'model>(BTreeMap<ItemId, CrateFiles<'model>>);

impl<'model> Consumers<'model> {
    #[must_use]
    pub fn files(&self, item: ItemId, root: &Path) -> Option<&BTreeSet<&'model Path>> {
        self.0.get(&item).and_then(|crates| crates.get(root))
    }
}

impl<'model, Entity> From<Placed<'model, Entity>> for Consumers<'model>
where
    Entity: Copy,
    Owned: From<Entity>,
{
    fn from(placed: Placed<'model, Entity>) -> Consumers<'model> {
        let Owned { reached, shipped, .. } = Owned::from(placed.entity());
        let file = placed.site().path();
        let root = placed.root();
        Consumers(shipped.map_or_default(|_| {
            reached.0.into_iter().map(|target| (target, BTreeMap::from([(root, BTreeSet::from([file]))]))).collect()
        }))
    }
}

impl<'model> FromIterator<Consumers<'model>> for Consumers<'model> {
    fn from_iter<Parts: IntoIterator<Item = Consumers<'model>>>(parts: Parts) -> Consumers<'model> {
        let found = parts
            .into_iter()
            .flat_map(|part| part.0)
            .flat_map(|(target, crates)| crates.into_iter().map(move |(root, files)| (target, root, files)));
        Consumers(found.fold(
            BTreeMap::new(),
            |mut consumers: BTreeMap<ItemId, CrateFiles<'model>>, (target, root, files)| {
                consumers.entry(target).or_default().entry(root).or_default().extend(files);
                consumers
            },
        ))
    }
}

#[derive(Debug, Default)]
pub(crate) struct Consumed(BTreeSet<ItemId>);

impl Consumed {
    pub(crate) fn within<'model, Entity>(
        files: &BTreeSet<&Path>,
        placed: impl Iterator<Item = Placed<'model, Entity>>,
    ) -> Consumed
    where
        Entity: Copy,
        Owned: From<Entity>,
    {
        placed
            .filter(|found| files.contains(found.site().path()))
            .map(|found| Consumed(Owned::from(found.entity()).reached.0))
            .collect()
    }

    pub(crate) fn roots<'model>(
        &self,
        items: impl Iterator<Item = Placed<'model, &'model ItemDeclaration>>,
    ) -> BTreeSet<&'model Path> {
        items.filter(|placed| self.0.contains(&placed.entity().declared())).map(Placed::root).collect()
    }
}

impl FromIterator<Consumed> for Consumed {
    fn from_iter<Parts: IntoIterator<Item = Consumed>>(parts: Parts) -> Consumed {
        Consumed(parts.into_iter().flat_map(|part| part.0).collect())
    }
}

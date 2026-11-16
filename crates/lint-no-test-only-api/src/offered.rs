use std::collections::BTreeSet;

use graph::{Exposure, Function, Held, Holding, Implementation, Input, ItemId, Member, Model, Named, Origin, Owner};
use graph::{Harness, Placed, Reach};
use graph::{Region, Resolved, Seat, Site, Sited, TypeUses};
use lint_core::Violation;
use lint_layout::{Name, Whole, Word, words};

use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Offer<'model> {
    member: Member,
    site: Site<'model>,
    krate: &'model Name,
}

impl<'model> Offer<'model> {
    pub(crate) fn every(model: &'model Model) -> Vec<Offer<'model>> {
        let constructors = model.functions().filter_map(Self::constructor);
        let derived: BTreeSet<ItemId> =
            model.implementations().filter_map(|placed| Self::derived(placed.entity())?.subject_item()).collect();
        let variants = model.holdings().filter_map(|placed| Self::variant(model, &derived, placed));
        constructors.chain(variants).collect()
    }

    pub(crate) fn shipped(self, support: &BTreeSet<Name>) -> Option<Offer<'model>> {
        support.get(self.krate).map_or(Some(self), |_| None)
    }

    pub(crate) fn tested_only(self, used: &Used) -> Option<Violation<'model>> {
        let shipped = used.shipped.get(&self.member).map_or(Some(()), |_| None);
        used.tested.get(&self.member).and(shipped).map(|()| Violation::new(self.site, Finding::OnlyTests))
    }

    fn derived(implementation: &Implementation) -> Option<&Implementation> {
        Some(implementation.origin()).filter(|origin| *origin == Origin::Derived)?;
        Some(AsRef::<Region>::as_ref(implementation).reach())
            .filter(|reach| *reach != Reach::Test)
            .map(|_| implementation)
    }

    fn constructor(placed: Placed<'model, &'model Function>) -> Option<Offer<'model>> {
        let function = placed.entity();
        let owner = AsRef::<Seat>::as_ref(function).owner()?;
        Offering::admits(*AsRef::<Exposure>::as_ref(function), *AsRef::<Region>::as_ref(function))?;
        Making::made_by(function, owner)?;
        let name: &Name = function.as_ref();
        Some(Offer { member: Member::new(owner, name.clone()), site: placed.site(), krate: placed.krate() })
    }

    fn variant(
        model: &'model Model,
        derived: &BTreeSet<ItemId>,
        placed: Placed<'model, &'model Holding>,
    ) -> Option<Offer<'model>> {
        let holding = placed.entity();
        let owner = Some(holding.held()).filter(|held| *held == Held::Variant).and(holding.declares())?;
        derived.get(&owner).map_or(Some(()), |_| None)?;
        Offering::admits(model.declared(owner)?.exposure(), *AsRef::<Region>::as_ref(holding))?;
        let name: &Name = holding.as_ref();
        Some(Offer { member: Member::new(owner, name.clone()), site: placed.site(), krate: placed.krate() })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Used {
    shipped: BTreeSet<Member>,
    tested: BTreeSet<Member>,
}

impl From<&Model> for Used {
    fn from(model: &Model) -> Used {
        model.mentions().fold(Used::default(), |mut used, placed| {
            let found = placed.entity();
            let member = found.resolved().member().cloned();
            used.reached(Ground::reaching(placed.harness(), *AsRef::<Region>::as_ref(found))).extend(member);
            used
        })
    }
}

impl Used {
    fn reached(&mut self, ground: Ground) -> &mut BTreeSet<Member> {
        match ground {
            Ground::Tests => &mut self.tested,
            Ground::Shipped => &mut self.shipped,
        }
    }
}

#[derive(Debug)]
struct Offering;

impl Offering {
    fn admits(exposure: Exposure, region: Region) -> Option<Reach> {
        Self::exposed(exposure).and(Some(region.reach()).filter(|reach| *reach != Reach::Test))
    }

    fn exposed(exposure: Exposure) -> Option<Exposure> {
        match exposure {
            Exposure::Public | Exposure::Restricted => Some(exposure),
            Exposure::Hidden => None,
        }
    }
}

#[derive(Debug)]
struct Making {
    owner: ItemId,
    found: Option<ItemId>,
}

impl Making {
    const SELF: [Word<'static>; 1] = words!["Self"];

    fn made_by(function: &Function, owner: ItemId) -> Option<ItemId> {
        Some(function.owner()).filter(|kind| *kind == Owner::Inherent)?;
        let received = function.signature().inputs().iter().find_map(Input::receiver);
        let output = received.map_or_else(|| function.signature().output().written(), |_| None)?;
        let mut making = Making { owner, found: None };
        output.each_use(&mut making);
        making.found
    }
}

impl TypeUses for Making {
    fn named(&mut self, named: &Named<Sited>) {
        let resolved: &Resolved = named.as_ref();
        let own = resolved.crate_item().filter(|item| *item == self.owner);
        let selfed = resolved.parameter().and_then(|name| name.matches::<Whole>(&Self::SELF)).map(|_| self.owner);
        self.found = self.found.or(own).or(selfed);
        named.each_inner_use(self);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ground {
    Shipped,
    Tests,
}

impl Ground {
    fn reaching(harness: Harness, region: Region) -> Ground {
        match harness {
            Harness::TestTarget => Ground::Tests,
            Harness::Elsewhere => Self::reached(region.reach()),
        }
    }

    fn reached(reach: Reach) -> Ground {
        match reach {
            Reach::Test => Ground::Tests,
            Reach::Shipped | Reach::Transcribed => Ground::Shipped,
        }
    }
}

mod identities;

use std::collections::{BTreeMap, BTreeSet};

use code_model::{Implementation, ItemId, Model, Origin, Placed, Resolved, Site};
use lint_core::Violation;
use lint_layout::Name;

use crate::finding::Finding;
use identities::{Identities, Role};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Derive<'model> {
    subject: ItemId,
    name: &'model Name,
    site: Site<'model>,
}

impl<'model> Derive<'model> {
    pub(crate) fn flagged(&self, owners: &BTreeSet<ItemId>) -> Option<Violation<'model>> {
        owners.get(&self.subject).map(|_| Violation::new(self.site, Finding::UnresolvedDerive(self.name)))
    }
}

#[derive(Clone, Copy, Debug)]
struct Played<'model> {
    role: Role,
    derive: Derive<'model>,
}

impl<'model> Played<'model> {
    fn recognised(
        model: &Model,
        identities: &Identities,
        placed: Placed<'model, &'model Implementation>,
    ) -> Option<Played<'model>> {
        let derived = Self::derived(placed.entity())?;
        let role = identities.role(model, derived.implemented()?)?;
        let subject: &Resolved = derived.subject_type().named()?.as_ref();
        let derive = Derive { subject: subject.crate_item()?, name: derived.subject(), site: placed.site() };
        Some(Played { role, derive })
    }

    fn derived(implementation: &Implementation) -> Option<&Implementation> {
        match implementation.origin() {
            Origin::Derived => Some(implementation),
            Origin::Written => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Kinds<'model> {
    validated: BTreeSet<ItemId>,
    plain: BTreeSet<ItemId>,
    unresolved: Vec<Derive<'model>>,
}

impl<'model> Kinds<'model> {
    pub(crate) fn read(model: &'model Model) -> Kinds<'model> {
        let identities = Identities::read(model);
        let played: BTreeMap<Role, Vec<Derive<'model>>> = model
            .implementations()
            .filter_map(|placed| Played::recognised(model, &identities, placed))
            .fold(BTreeMap::new(), |mut played, play| {
                played.entry(play.role).or_default().push(play.derive);
                played
            });
        let derives = |role: Role| played.get(&role).cloned().unwrap_or_default();
        let subjects = |role: Role| -> BTreeSet<ItemId> { derives(role).iter().map(|derive| derive.subject).collect() };
        let validated = subjects(Role::DeserializesChecked);
        let plain = subjects(Role::Deserializes).difference(&validated).copied().collect();
        Kinds { validated, plain, unresolved: derives(Role::Unresolved) }
    }

    pub(crate) fn unvalidated(&self, item: ItemId) -> Option<ItemId> {
        match self.validated.get(&item) {
            Some(_) => None,
            None => Some(item),
        }
    }

    pub(crate) fn plain(&self) -> &BTreeSet<ItemId> {
        &self.plain
    }

    pub(crate) fn unresolved(&self) -> &[Derive<'model>] {
        &self.unresolved
    }
}

mod identities;

use std::collections::{BTreeMap, BTreeSet};

use code_model::{Implementation, ItemId, Model, Origin, Placed, Resolved, Site};
use lint_layout::Name;

use identities::{Identities, Role};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Derive<'model> {
    subject: ItemId,
    name: &'model Name,
    site: Site<'model>,
}

impl<'model> Derive<'model> {
    pub(crate) fn subject(&self) -> ItemId {
        self.subject
    }

    pub(crate) fn name(&self) -> &'model Name {
        self.name
    }

    pub(crate) fn site(&self) -> Site<'model> {
        self.site
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
        let mut played: BTreeMap<Role, Vec<Derive<'model>>> = model
            .implementations()
            .filter_map(|placed| Self::played(&identities, placed))
            .fold(BTreeMap::new(), |mut played, (role, derive)| {
                played.entry(role).or_default().push(derive);
                played
            });
        let mut subjects = |role: Role| -> BTreeSet<ItemId> {
            played.remove(&role).unwrap_or_default().iter().map(Derive::subject).collect()
        };
        let validated = subjects(Role::DeserializesChecked);
        let plain = subjects(Role::Deserializes).difference(&validated).copied().collect();
        let unresolved = played.remove(&Role::Unresolved).unwrap_or_default();
        Kinds { validated, plain, unresolved }
    }

    pub(crate) fn unvalidated(&self, item: ItemId) -> Option<ItemId> {
        match self.validated.get(&item) {
            Some(_) => None,
            None => Some(item),
        }
    }

    pub(crate) fn plain(&self, item: ItemId) -> Option<ItemId> {
        self.plain.get(&item).copied()
    }

    pub(crate) fn unresolved(&self) -> &[Derive<'model>] {
        &self.unresolved
    }

    fn played(
        identities: &Identities,
        placed: Placed<'model, &'model Implementation>,
    ) -> Option<(Role, Derive<'model>)> {
        let implementation = placed.entity();
        let derived = match implementation.origin() {
            Origin::Derived => Some(implementation),
            Origin::Written => None,
        }?;
        let role = identities.role(derived.implemented()?)?;
        let subject: &Resolved = derived.subject_type().named()?.as_ref();
        Some((role, Derive { subject: subject.crate_item()?, name: derived.subject(), site: placed.site() }))
    }
}

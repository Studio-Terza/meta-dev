mod identities;

use std::collections::{BTreeMap, BTreeSet};

use code_model::{Holding, Implementation, ItemId, Model, Origin, Resolved};

use identities::{Identities, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reading {
    Plain,
    Unresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Part {
    role: Role,
    origin: Origin,
}

#[derive(Debug)]
struct Played {
    part: Part,
    subject: ItemId,
}

#[derive(Debug)]
pub(crate) struct Kinds {
    validated: BTreeSet<ItemId>,
    owners: BTreeMap<ItemId, Reading>,
}

impl Kinds {
    pub(crate) fn read(model: &Model) -> Kinds {
        let identities = Identities::read(model);
        let played: Vec<Played> =
            model.implementations().filter_map(|placed| Self::played(&identities, placed.entity())).collect();
        let derived = |role: Role| -> BTreeSet<ItemId> {
            let part = Part { role, origin: Origin::Derived };
            played.iter().filter(|played| played.part == part).map(|played| played.subject).collect()
        };
        let validated = derived(Role::DeserializesChecked);
        let read: BTreeSet<ItemId> = derived(Role::Deserializes).union(&validated).copied().collect();
        let plain = read.difference(&validated).map(|item| (*item, Reading::Plain));
        let unresolved = derived(Role::Unresolved);
        let unread = unresolved.difference(&read).map(|item| (*item, Reading::Unresolved));
        let owners = plain.chain(unread).collect();
        Kinds { validated, owners }
    }

    pub(crate) fn unvalidated(&self, item: ItemId) -> Option<ItemId> {
        match self.validated.get(&item) {
            Some(_) => None,
            None => Some(item),
        }
    }

    pub(crate) fn reading(&self, holding: &Holding) -> Option<Reading> {
        holding.declares().and_then(|item| self.owners.get(&item).copied())
    }

    fn played(identities: &Identities, implementation: &Implementation) -> Option<Played> {
        let role = identities.role(implementation.implemented()?)?;
        let subject: &Resolved = implementation.subject_type().named()?.as_ref();
        Some(Played { part: Part { role, origin: implementation.origin() }, subject: subject.crate_item()? })
    }
}

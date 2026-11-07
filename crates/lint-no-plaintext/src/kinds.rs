mod identities;

use std::collections::BTreeSet;

use code_model::{Held, Holding, Implementation, ItemId, Model, Origin, Placed, Resolved};

use identities::{Identities, Role};

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
    plain_owners: BTreeSet<ItemId>,
}

impl Kinds {
    pub(crate) fn read(model: &Model) -> Kinds {
        let identities = Identities::read(model);
        let played: Vec<Played> =
            model.implementations().filter_map(|placed| Self::played(&identities, placed.entity())).collect();
        let subjects = |role: Role, origin: Origin| {
            let part = Part { role, origin };
            played.iter().filter(move |played| played.part == part).map(|played| played.subject)
        };
        let deserialized: BTreeSet<ItemId> = subjects(Role::Deserializes, Origin::Derived).collect();
        let converting: BTreeSet<ItemId> = subjects(Role::Converts, Origin::Written).collect();
        let converted = model
            .holdings()
            .map(Placed::entity)
            .filter(|holding| holding.held() == Held::TryFromSource)
            .filter_map(Holding::declares)
            .filter_map(|item| converting.get(&item).copied());
        let validated: BTreeSet<ItemId> = subjects(Role::Checks, Origin::Written).chain(converted).collect();
        let plain_owners = deserialized.difference(&validated).copied().collect();
        Kinds { validated, plain_owners }
    }

    pub(crate) fn unvalidated(&self, item: ItemId) -> Option<ItemId> {
        match self.validated.get(&item) {
            Some(_) => None,
            None => Some(item),
        }
    }

    pub(crate) fn plain_owner(&self, holding: &Holding) -> Option<ItemId> {
        holding.declares().and_then(|item| self.plain_owners.get(&item).copied())
    }

    fn played(identities: &Identities, implementation: &Implementation) -> Option<Played> {
        let role = identities.role(implementation.implemented()?)?;
        let subject: &Resolved = implementation.subject_type().named()?.as_ref();
        Some(Played { part: Part { role, origin: implementation.origin() }, subject: subject.crate_item()? })
    }
}

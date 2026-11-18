mod stored;

use std::collections::BTreeMap;

use graph::{Function, Implementation, ItemId, ItemKind, Model, Owner, Reach, Region, Resolved, Seat};
use lint_core::Site;
use lint_layout::{Name, Whole, Word, words};

use super::exposure::Exposure;
use super::profile::Profile;
use crate::subject::Subject;
use stored::Stores;

#[derive(Clone, Copy, Debug)]
struct Typed<'model> {
    krate: &'model Name,
    name: &'model Name,
    site: Site<'model>,
    exposure: Exposure,
}

#[derive(Debug)]
pub(super) struct Reading<'model> {
    model: &'model Model,
    types: BTreeMap<ItemId, Typed<'model>>,
    stores: Stores,
}

impl<'model> From<&'model Model> for Reading<'model> {
    fn from(model: &'model Model) -> Reading<'model> {
        let types = model
            .items()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter_map(|placed| {
                let id = placed.entity().declared();
                let item = model.declared(id).filter(|found| Self::TYPES.contains(&found.kind()))?;
                let exposure = Exposure::from(item.exposure());
                Some((id, Typed { krate: placed.krate(), name: item.as_ref(), site: placed.site(), exposure }))
            })
            .collect();
        Reading { model, types, stores: Stores::from(model) }
    }
}

impl<'model> Reading<'model> {
    const TYPES: [ItemKind; 2] = [ItemKind::Struct, ItemKind::Enum];
    const TRAITS: [Word<'static>; 5] = words!["Display", "FromStr", "Serialize", "Deserialize", "Record"];
    const DERIVED: [Word<'static>; 3] = words!["Serialize", "Deserialize", "Record"];
    const METHODS: [Word<'static>; 2] = words!["encode", "decode"];
    const LEADS: [Word<'static>; 2] = words!["to", "from"];
    const OWNERS: [Owner; 2] = [Owner::Inherent, Owner::TraitImpl];

    pub(super) fn profiles(&self) -> Vec<Profile<'model>> {
        let declared = self
            .types
            .values()
            .map(|typed| Profile::declared(Self::subject(*typed), typed.site, typed.exposure, Vec::new()));
        let implemented = self
            .model
            .implementations()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter_map(|placed| self.implemented(placed.entity()));
        let methods = self
            .model
            .functions()
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter(|placed| Self::OWNERS.contains(&placed.entity().owner()))
            .filter_map(|placed| self.method(placed.entity()));
        declared.chain(implemented).chain(methods).collect()
    }

    fn implemented(&self, implementation: &Implementation) -> Option<Profile<'model>> {
        let resolved: &Resolved = implementation.subject_type().named()?.as_ref();
        let typed = self.types.get(&resolved.crate_item()?)?;
        let listed = implementation.origin().sorted(&Self::TRAITS[..], &Self::DERIVED[..]);
        let name = implementation.interface_name().filter(|found| found.matches::<Whole>(listed).is_some())?;
        Stores::record(name).map_or(Some(()), |()| self.stores.admits(implementation))?;
        Some(Profile::declared(Self::subject(*typed), typed.site, Exposure::Private, vec![name.clone()]))
    }

    fn method(&self, function: &Function) -> Option<Profile<'model>> {
        let typed = self.types.get(&AsRef::<Seat>::as_ref(function).owner()?)?;
        let name: &Name = function.as_ref();
        let words = name.words();
        name.matches::<Whole>(&Self::METHODS)
            .map(|_| ())
            .or_else(|| words.first().and_then(|first| first.matches::<Whole>(&Self::LEADS)).map(|_| ()))?;
        Some(Profile::declared(Self::subject(*typed), typed.site, Exposure::Private, vec![name.clone()]))
    }

    fn subject(typed: Typed<'model>) -> Subject<'model> {
        Subject::new(typed.krate, typed.name.clone())
    }
}

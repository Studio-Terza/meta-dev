use std::collections::{BTreeMap, BTreeSet};

use code_model::{ItemKind, ItemPath, Model, PathUse, Placed, Reach, Region, Seat};
use lint_core::Violation;
use lint_effect_sites::OutsideFillers;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
struct Filler<'model> {
    krate: &'model Name,
    name: Name,
}

#[derive(Debug, Default)]
pub(crate) struct Ports<'model>(BTreeMap<&'model Name, BTreeSet<Name>>);

impl<'model> FromIterator<Filler<'model>> for Ports<'model> {
    fn from_iter<Fillers: IntoIterator<Item = Filler<'model>>>(fillers: Fillers) -> Ports<'model> {
        fillers.into_iter().fold(Ports::default(), |mut ports, filler| {
            ports.0.entry(filler.krate).or_default().insert(filler.name);
            ports
        })
    }
}

impl<'model> From<&'model Model> for Ports<'model> {
    fn from(model: &'model Model) -> Ports<'model> {
        let marks: Vec<ItemPath> = [Self::PORT.as_slice(), Self::QUALIFIED.as_slice()]
            .into_iter()
            .map(|words| words.iter().copied().collect())
            .collect();
        model
            .items()
            .filter_map(|placed| {
                let item = model.declared(*placed.entity()).filter(|found| found.kind() == ItemKind::Trait)?;
                item.marks().iter().find(|mark| marks.contains(*mark))?;
                Some(Filler { krate: placed.krate(), name: AsRef::<Name>::as_ref(item).suffixed(Self::FILLER) })
            })
            .collect()
    }
}

impl<'model> Ports<'model> {
    const PORT: [Word<'static>; 1] = words!["port"];
    const QUALIFIED: [Word<'static>; 2] = words!["port_core", "port"];
    const FILLER: Word<'static> = Word::new("Filler");
    const RELEASE: [Word<'static>; 1] = words!["Release"];

    pub(crate) fn stray_effects(&self, model: &'model Model) -> Vec<Violation<'model>> {
        let outside: OutsideFillers<'model> = model.uses().filter(|placed| self.stray(*placed).is_some()).collect();
        Vec::from(outside)
    }

    fn stray(&self, placed: Placed<'model, &'model PathUse>) -> Option<()> {
        let fillers = self.0.get(placed.krate())?;
        let used = placed.entity();
        Some(AsRef::<Region>::as_ref(used).reach()).filter(|reach| *reach == Reach::Shipped)?;
        let implemented = AsRef::<Seat>::as_ref(used).implemented();
        implemented.and_then(|name| Self::filled(fillers, name)).map_or(Some(()), |()| None)
    }

    fn filled(fillers: &BTreeSet<Name>, name: &Name) -> Option<()> {
        fillers.get(name).map(|_| ()).or_else(|| name.matches::<Whole>(&Self::RELEASE).map(|_| ()))
    }
}

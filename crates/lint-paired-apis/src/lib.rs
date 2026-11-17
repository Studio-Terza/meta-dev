mod declared;
mod pairs;

use std::collections::BTreeMap;

use graph::{Exposure, Function, ItemId, Model, Owner, Placed, Reach, Region, Seat};
use lint_core::{Headline, Report};
use lint_file::ModelLint;
use lint_layout::Name;

use declared::Declared;
use pairs::Pairs;

#[derive(Debug)]
pub struct PairedApis;

#[derive(Debug)]
struct Owned<'model> {
    owner: ItemId,
    declared: Declared<'model>,
}

impl ModelLint for PairedApis {
    fn report(model: &Model) -> Report<'_> {
        let owned = model.functions().filter_map(Owned::placed).fold(BTreeMap::new(), Owned::grouped);
        Report::new(PairedApis::HEADLINE, Pairs::listed(owned.values()))
    }
}

impl PairedApis {
    const HEADLINE: Headline = Headline::new(
        "report only, a paired API leaves the ending to every caller; write a guard whose Drop does the ending",
    );
}

impl<'model> Owned<'model> {
    fn placed(placed: Placed<'model, &'model Function>) -> Option<Owned<'model>> {
        let function = placed.entity();
        let seat: &Seat = function.as_ref();
        let region: &Region = function.as_ref();
        let owner = Self::paired(function).filter(|_| region.reach() == Reach::Shipped).and(seat.owner())?;
        let name: &Name = function.as_ref();
        Some(Owned { owner, declared: Declared::new(name.clone(), placed.site()) })
    }

    fn paired(function: &Function) -> Option<&Function> {
        match function.owner() {
            Owner::Inherent => (*AsRef::<Exposure>::as_ref(function) == Exposure::Public).then_some(function),
            Owner::TraitDecl => Some(function),
            Owner::Free | Owner::TraitImpl | Owner::Nested | Owner::Foreign => None,
        }
    }

    fn grouped(
        owners: BTreeMap<ItemId, Vec<Declared<'model>>>,
        owned: Owned<'model>,
    ) -> BTreeMap<ItemId, Vec<Declared<'model>>> {
        let mut grown = owners;
        grown.entry(owned.owner).or_default().push(owned.declared);
        grown
    }
}

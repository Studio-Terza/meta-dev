mod finding;

use graph::{Function, ItemKind, Model, Owner, Placed, Primitive, Resolved, Seat};
use lint_core::{Headline, Report, Violation};
use lint_file::ModelLint;
use lint_layout::Name;

use finding::Finding;

#[derive(Debug)]
pub struct EnumPredicates;

impl ModelLint for EnumPredicates {
    fn report(model: &Model) -> Report<'_> {
        let found = model.functions().filter_map(|placed| Self::predicate(model, placed)).collect();
        Report::new(Self::HEADLINE, found)
    }
}

impl EnumPredicates {
    const HEADLINE: Headline = Headline::new(
        "an enum method returning bool throws away the variant; return an enum or match where the answer is used",
    );

    fn predicate<'model>(model: &'model Model, placed: Placed<'model, &'model Function>) -> Option<Violation<'model>> {
        let function = placed.entity();
        let seat: &Seat = function.as_ref();
        let owner = Some(function).filter(|candidate| candidate.owner() == Owner::Inherent).and(seat.owner())?;
        model.declared(owner).filter(|item| item.kind() == ItemKind::Enum)?;
        let returned = function.signature().output().written()?.named()?;
        AsRef::<Resolved>::as_ref(returned).primitive().filter(|primitive| *primitive == Primitive::Bool)?;
        let name: &Name = function.as_ref();
        Some(Violation::new(placed.site(), Finding::BoolPredicate { name: name.clone() }))
    }
}

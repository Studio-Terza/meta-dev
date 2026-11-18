use std::collections::BTreeSet;

use algebra::{Monoid, Semigroup};
use syntax::{ExprMethodCall, ExprPath, ItemMod, TraitBound};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind};

#[derive(Debug, Default)]
pub struct Consumers {
    bounds: BTreeSet<Name>,
    called: BTreeSet<Name>,
}

#[derive(Debug)]
pub struct Consuming {
    kinds: &'static [Kind],
}

impl From<&SourceFile> for Consuming {
    fn from(file: &SourceFile) -> Consuming {
        let kinds = file
            .shipped()
            .map_or(&[][..], |_| &[Kind::ItemMod, Kind::TraitBound, Kind::ExprMethodCall, Kind::ExprPath][..]);
        Consuming { kinds }
    }
}

impl Gathers<Consumers> for Contributing<Consuming> {
    fn gathered(self, file: &SourceFile) -> Consumers {
        self.concluded(file, |_, (), consumers| consumers)
    }
}

impl FromIterator<Consumers> for Consumers {
    fn from_iter<Parts: IntoIterator<Item = Consumers>>(parts: Parts) -> Consumers {
        Monoid::concat(parts)
    }
}

impl Semigroup for Consumers {
    fn combine(mut self, other: Consumers) -> Consumers {
        self.bounds.extend(other.bounds);
        self.called.extend(other.called);
        self
    }
}

impl Monoid for Consumers {}

impl Consumers {
    pub(crate) fn consuming<'name>(&'name self, name: &'name Name, provided: &'name [Name]) -> Option<&'name Name> {
        self.bounds.get(name).or_else(|| provided.iter().find(|method| self.called.contains(*method)))
    }

    fn called(called: Option<Name>) -> Contributed<(), Consumers> {
        Contributed::parted(Consumers { bounds: BTreeSet::new(), called: called.into_iter().collect() })
    }
}

impl Contribute for Consuming {
    type Open = ();
    type Part = Consumers;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Consumers> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_trait_bound(
        &self,
        bound: &TraitBound,
        _: &Facts<'_, TraitBound>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Consumers> {
        let bounds = bound.path.segments.last().map(|segment| Name::from(&segment.ident)).into_iter().collect();
        Contributed::parted(Consumers { bounds, called: BTreeSet::new() })
    }

    fn on_expr_method_call(
        &self,
        _: &ExprMethodCall,
        facts: &Facts<'_, ExprMethodCall>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Consumers> {
        Consumers::called(facts.name().cloned())
    }

    fn on_expr_path(
        &self,
        path: &ExprPath,
        _: &Facts<'_, ExprPath>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Consumers> {
        Consumers::called(
            path.path
                .segments
                .iter()
                .nth(1)
                .and_then(|_| path.path.segments.last())
                .map(|segment| Name::from(&segment.ident)),
        )
    }
}

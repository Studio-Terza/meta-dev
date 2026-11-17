use std::collections::BTreeSet;

use syntax::{ExprMethodCall, ExprPath, ItemMod, TraitBound};

use lint_core::SourceFile;
use lint_layout::Name;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

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

impl FromIterator<Consumers> for Consumers {
    fn from_iter<Parts: IntoIterator<Item = Consumers>>(parts: Parts) -> Consumers {
        parts.into_iter().fold(Consumers::default(), |mut merged, part| {
            merged.bounds.extend(part.bounds);
            merged.called.extend(part.called);
            merged
        })
    }
}

impl Consumers {
    pub(crate) fn consuming<'name>(&'name self, name: &'name Name, provided: &'name [Name]) -> Option<&'name Name> {
        self.bounds.get(name).or_else(|| provided.iter().find(|method| self.called.contains(*method)))
    }
}

impl Fold for Consuming {
    type Found = Consumers;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&self, found: Consumers, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Consumers> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_trait_bound(&self, found: Consumers, bound: &TraitBound, _: &Facts<'_, TraitBound>) -> Visit<Consumers> {
        Visit::descending(Consumers {
            bounds: found
                .bounds
                .into_iter()
                .chain(bound.path.segments.last().map(|segment| Name::from(&segment.ident)))
                .collect(),
            ..found
        })
    }

    fn on_expr_method_call(
        &self,
        found: Consumers,
        _: &ExprMethodCall,
        facts: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Consumers> {
        Visit::descending(Consumers {
            called: found.called.into_iter().chain(facts.name().cloned()).collect(),
            ..found
        })
    }

    fn on_expr_path(&self, found: Consumers, path: &ExprPath, _: &Facts<'_, ExprPath>) -> Visit<Consumers> {
        let called = path
            .path
            .segments
            .iter()
            .nth(1)
            .and_then(|_| path.path.segments.last())
            .map(|segment| Name::from(&segment.ident));
        Visit::descending(Consumers { called: found.called.into_iter().chain(called).collect(), ..found })
    }
}

use std::collections::BTreeSet;

use syn::{ExprMethodCall, ExprPath, ItemMod, TraitBound};

use lint_core::{SourceFile, Walk};
use lint_layout::Name;
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};

#[derive(Debug, Default)]
pub struct Consumers {
    bounds: BTreeSet<Name>,
    called: BTreeSet<Name>,
}

#[derive(Debug)]
pub struct Consuming {
    consumers: Consumers,
    kinds: &'static [Kind],
}

impl From<&SourceFile> for Consuming {
    fn from(file: &SourceFile) -> Consuming {
        let kinds = file
            .shipped()
            .map_or(&[][..], |_| &[Kind::ItemMod, Kind::TraitBound, Kind::ExprMethodCall, Kind::ExprPath][..]);
        Consuming { consumers: Consumers::default(), kinds }
    }
}

impl Gathers<Consumers> for Consuming {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Consumers {
        self.consumers
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

impl<'ast> Watch<'ast> for Consuming {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_trait_bound(&mut self, bound: &'ast TraitBound) -> Descend {
        self.consumers.bounds.extend(bound.path.segments.last().map(|segment| Name::from(&segment.ident)));
        Descend::Into
    }

    fn on_expr_method_call(&mut self, call: &'ast ExprMethodCall) -> Descend {
        self.consumers.called.insert(Name::from(&call.method));
        Descend::Into
    }

    fn on_expr_path(&mut self, path: &'ast ExprPath) -> Descend {
        self.consumers.called.extend(
            path.path
                .segments
                .iter()
                .nth(1)
                .and_then(|_| path.path.segments.last())
                .map(|segment| Name::from(&segment.ident)),
        );
        Descend::Into
    }
}

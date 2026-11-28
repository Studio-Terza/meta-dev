use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use proc_macro2::Ident;
use syn::{ItemImpl, ItemTrait, ItemUse, Macro, Type};

use pipeline::Measure;

use crate::finding::{Finding, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Fact, Lint, Reads};
use crate::source::{Place, Source};
use crate::use_leaf::UseLeaf;
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum AdapterFact {
    Real { crate_name: String, name: String },
    Named { crate_name: String, name: String, path: Arc<Path>, line: usize },
    Port { crate_name: String, name: String },
    Doubled { crate_name: String, name: String, path: Arc<Path>, line: usize },
}

#[derive(Debug, Measure)]
pub(crate) struct AdapterFacts(Vec<AdapterFact>);

#[derive(Debug)]
pub(crate) struct RealAdapters<'file> {
    source: &'file Source,
    admitted: Admitted,
    imported: BTreeMap<String, Imported>,
    facts: Vec<AdapterFact>,
}

#[derive(Clone, Debug)]
struct Imported {
    crate_name: String,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Admitted {
    Everything,
    Own(String),
    Nothing,
}

impl RealAdapters<'_> {
    const MARK: &'static str = "Real";
    const PORT: &'static str = "port";
    const FILLER: &'static str = "Filler";
    const FIX: &'static str = "build over a double here, and wire the real adapter in a binary's main.rs";
    const DOUBLED: &'static str = "record the input and feed it through the production pipeline";

    fn doubled(&self, item: &ItemImpl) -> Option<AdapterFact> {
        (self.source.place() == Place::Tests).then_some(())?;
        let (_negated, trait_path, _for) = item.trait_.as_ref()?;
        let last = trait_path.segments.last().map(|segment| &segment.ident)?;
        let written = match trait_path.segments.first().map(|segment| &segment.ident) {
            Some(first) if trait_path.segments.len() > 1 => {
                Some(Imported { crate_name: first.to_string(), name: last.to_string() })
            }
            Some(_) | None => self.imported.get(&last.to_string()).cloned(),
        }?;
        (written.crate_name != self.source.package().crate_name()).then(|| AdapterFact::Doubled {
            crate_name: written.crate_name,
            name: written.name,
            path: Arc::clone(self.source.shown()),
            line: last.span().start().line,
        })
    }

    fn marked(item: &ItemImpl) -> Option<&Ident> {
        let (_negated, trait_path, _for) = item.trait_.as_ref()?;
        trait_path.segments.last().filter(|mark| mark.ident == RealAdapters::MARK)?;
        match item.self_ty.as_ref() {
            Type::Path(adapter) => adapter.path.segments.last().map(|segment| &segment.ident),
            _ => None,
        }
    }

    fn named(&self, first: Option<&Ident>, name: &Ident) -> Option<AdapterFact> {
        let crate_name = first?.to_string();
        let admitted = match &self.admitted {
            Admitted::Everything => Some(()),
            Admitted::Own(own) => (*own == crate_name).then_some(()),
            Admitted::Nothing => None,
        };
        admitted.xor(Some(())).map(|()| AdapterFact::Named {
            crate_name,
            name: name.to_string(),
            path: Arc::clone(self.source.shown()),
            line: name.span().start().line,
        })
    }
}

impl Hooks for RealAdapters<'_> {
    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        let real = self.source.ported().and_then(|()| RealAdapters::marked(item));
        let fact = real.map(|adapter| AdapterFact::Real {
            crate_name: self.source.package().crate_name(),
            name: adapter.to_string(),
        });
        let doubled = self.doubled(item);
        self.facts.extend(fact.into_iter().chain(doubled));
        Descent::Into
    }

    fn item_trait(&mut self, item: &ItemTrait) -> Descent {
        let crate_name = self.source.package().crate_name();
        let declared = item.ident.to_string();
        let ported = item.attrs.iter().any(|attribute| {
            attribute.path().segments.last().is_some_and(|segment| segment.ident == RealAdapters::PORT)
        });
        let filled = ported.then(|| format!("{declared}{}", RealAdapters::FILLER));
        let ports = self.source.ported().map(|()| [Some(declared), filled].into_iter().flatten()).into_iter().flatten();
        self.facts.extend(ports.map(|name| AdapterFact::Port { crate_name: crate_name.clone(), name }));
        Descent::Into
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let leaves = UseLeaf::of_tree(&item.tree);
        let named: Vec<AdapterFact> = leaves
            .iter()
            .filter_map(|leaf| leaf.end().original().and_then(|name| self.named(leaf.path().first().copied(), name)))
            .collect();
        self.facts.extend(named);
        let imported = leaves.iter().filter_map(|leaf| {
            let crate_name = leaf.path().first()?.to_string();
            let name = leaf.end().original()?.to_string();
            Some((leaf.end().bound()?.to_string(), Imported { crate_name, name }))
        });
        self.imported.extend(imported);
        Descent::Over
    }

    fn path(&mut self, path: &syn::Path) -> Descent {
        let first = path.segments.first().map(|segment| &segment.ident);
        let named = path.segments.iter().nth(1).and_then(|second| self.named(first, &second.ident));
        self.facts.extend(named);
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Surface
    }
}

impl<'file> Lint<'file> for RealAdapters<'file> {
    const HEADLINE: &'static str = "a port's real adapter is named only in a binary's main.rs and in its own port crate's tests, and only that crate's tests implement its traits";
    const READS: Reads = Reads::Every;

    type Fact = AdapterFacts;

    fn new(source: &'file Source) -> RealAdapters<'file> {
        let admitted = match (source.binary_root(), source.place()) {
            (Some(()), _) => Admitted::Everything,
            (None, Place::Tests) => Admitted::Own(source.package().crate_name()),
            (None, Place::Shipped | Place::Elsewhere) => Admitted::Nothing,
        };
        RealAdapters { source, admitted, imported: BTreeMap::new(), facts: Vec::new() }
    }

    fn fact(self, _: &'file Source) -> AdapterFacts {
        AdapterFacts(self.facts)
    }
}

impl Fact for AdapterFacts {
    fn findings(self, _: &[Arc<Package>]) -> Vec<Finding> {
        let facts = self.0;
        let real: BTreeSet<(&str, &str)> = facts
            .iter()
            .filter_map(|fact| match fact {
                AdapterFact::Real { crate_name, name } => Some((crate_name.as_str(), name.as_str())),
                AdapterFact::Named { .. } | AdapterFact::Port { .. } | AdapterFact::Doubled { .. } => None,
            })
            .collect();
        let ports: BTreeSet<(&str, &str)> = facts
            .iter()
            .filter_map(|fact| match fact {
                AdapterFact::Port { crate_name, name } => Some((crate_name.as_str(), name.as_str())),
                AdapterFact::Named { .. } | AdapterFact::Real { .. } | AdapterFact::Doubled { .. } => None,
            })
            .collect();
        facts
            .iter()
            .filter_map(|fact| match fact {
                AdapterFact::Named { crate_name, name, path, line } => {
                    real.get(&(crate_name.as_str(), name.as_str())).map(|_real| {
                        Found::at(
                            *line,
                            format_args!("names {crate_name}::{name}, a port's real adapter"),
                            RealAdapters::FIX,
                        )
                        .in_file(RealAdapters::HEADLINE, path)
                    })
                }
                AdapterFact::Doubled { crate_name, name, path, line } => {
                    ports.get(&(crate_name.as_str(), name.as_str())).map(|_port| {
                        let said =
                            format_args!("a test implements {crate_name}::{name}, a port's trait, outside its crate");
                        Found::at(*line, said, RealAdapters::DOUBLED).in_file(RealAdapters::HEADLINE, path)
                    })
                }
                AdapterFact::Real { .. } | AdapterFact::Port { .. } => None,
            })
            .collect()
    }
}

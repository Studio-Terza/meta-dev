use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use proc_macro2::{Ident, TokenTree};
use syn::{Attribute, ItemStruct, ItemUse, Macro, Meta};

use crate::finding::{Finding, Found, Rule};
use crate::graph::Graph;
use crate::source::{Place, Source};
use crate::use_leaf::UseLeaf;
use crate::walk::{Descent, Expanded, Hooks};
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum AdapterFact {
    Real { crate_name: String, name: String },
    Named { crate_name: String, name: String, path: Arc<Path>, line: usize },
}

#[derive(Debug)]
pub(crate) struct RealAdapters<'file> {
    source: &'file Source,
    admitted: Admitted,
    facts: Vec<AdapterFact>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Admitted {
    Everything,
    Own(String),
    Nothing,
}

impl RealAdapters<'_> {
    const DERIVE: &'static str = "derive";
    const MARK: &'static str = "Real";
    const FIX: &'static str = "build over a double here, and wire the real adapter in a binary's main.rs";

    fn marked(attribute: &Attribute) -> Option<()> {
        let list = match &attribute.meta {
            Meta::List(list) if list.path.is_ident(RealAdapters::DERIVE) => Some(list),
            Meta::List(_) | Meta::Path(_) | Meta::NameValue(_) => None,
        }?;
        let mark = list
            .tokens
            .clone()
            .into_iter()
            .find(|tree| matches!(tree, TokenTree::Ident(ident) if ident == RealAdapters::MARK));
        mark.map(|_mark| ())
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
    fn item_struct(&mut self, item: &ItemStruct) -> Descent {
        let real = self.source.ported().and(item.attrs.iter().find_map(RealAdapters::marked));
        let fact = real.map(|()| AdapterFact::Real {
            crate_name: self.source.package().crate_name(),
            name: item.ident.to_string(),
        });
        self.facts.extend(fact);
        Descent::Into
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let named: Vec<AdapterFact> = UseLeaf::of_tree(&item.tree)
            .iter()
            .filter_map(|leaf| leaf.end().original().and_then(|name| self.named(leaf.path().first().copied(), name)))
            .collect();
        self.facts.extend(named);
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

impl<'file> Graph<'file> for RealAdapters<'file> {
    type Fact = AdapterFact;

    fn new(source: &'file Source) -> RealAdapters<'file> {
        let admitted = match (source.binary_root(), source.place()) {
            (Some(()), _) => Admitted::Everything,
            (None, Place::Tests) => Admitted::Own(source.package().crate_name()),
            (None, Place::Shipped | Place::Elsewhere) => Admitted::Nothing,
        };
        RealAdapters { source, admitted, facts: Vec::new() }
    }

    fn facts(self) -> Vec<AdapterFact> {
        self.facts
    }

    fn reduce(facts: Vec<AdapterFact>, _: &[Arc<Package>]) -> Vec<Finding> {
        let real: BTreeSet<(&str, &str)> = facts
            .iter()
            .filter_map(|fact| match fact {
                AdapterFact::Real { crate_name, name } => Some((crate_name.as_str(), name.as_str())),
                AdapterFact::Named { .. } => None,
            })
            .collect();
        facts
            .iter()
            .filter_map(|fact| match fact {
                AdapterFact::Named { crate_name, name, path, line } => {
                    real.get(&(crate_name.as_str(), name.as_str())).map(|_real| {
                        Found::at(*line, format_args!("names {crate_name}::{name}, a port's real adapter"), Self::FIX)
                            .in_file(Rule::RealAdapters, path)
                    })
                }
                AdapterFact::Real { .. } => None,
            })
            .collect()
    }
}

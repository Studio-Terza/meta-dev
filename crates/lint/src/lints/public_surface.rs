use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use proc_macro2::Ident;
use syn::{Item, ItemUse, Macro, Visibility};

use crate::finding::{Finding, Found, Rule};
use crate::graph::Graph;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::source::{Place, Source};
use crate::use_leaf::UseLeaf;
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum SurfaceFact {
    Export { crate_name: String, name: String, path: Arc<Path>, line: usize },
    Sighting { crate_name: String, reached: Reached, reader: Reader },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reached {
    Named(String),
    Glob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Reader {
    Shipped,
    Tests,
    Neither,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Used {
    #[default]
    Unused,
    Named,
    Globbed,
}

#[derive(Debug, Default)]
struct Reach {
    names: BTreeSet<String>,
    used: Used,
}

#[derive(Debug)]
pub(crate) struct PublicSurface<'file> {
    source: &'file Source,
    reader: Reader,
    own_crate: Option<String>,
    facts: Vec<SurfaceFact>,
}

impl Reach {
    fn absorb(&mut self, reached: &Reached, reader: Reader) {
        let used = match (reached, reader) {
            (_, Reader::Tests | Reader::Neither) => Used::Unused,
            (Reached::Named(_), Reader::Shipped) => Used::Named,
            (Reached::Glob, Reader::Shipped) => Used::Globbed,
        };
        self.used = self.used.max(used);
        self.names.extend(match reached {
            Reached::Named(name) => Some(name.clone()),
            Reached::Glob => None,
        });
    }
}

impl PublicSurface<'_> {
    fn sighting(&self, first: Option<&Ident>, reached: Reached) -> Option<SurfaceFact> {
        let foreign = first.map(Ident::to_string).filter(|named| Some(named) != self.own_crate.as_ref());
        foreign.map(|crate_name| SurfaceFact::Sighting { crate_name, reached, reader: self.reader })
    }

    fn exports(&mut self, items: &[Item]) {
        let crate_name = self.source.package().crate_name();
        let path = self.source.shown();
        let exported = items.iter().flat_map(|item| match item {
            Item::Use(used) if matches!(used.vis, Visibility::Public(_)) => {
                UseLeaf::of_tree(&used.tree).into_iter().filter_map(|leaf| leaf.end().bound()).collect()
            }
            Item::Mod(module) if matches!(module.vis, Visibility::Public(_)) => vec![&module.ident],
            _ => Vec::new(),
        });
        let facts: Vec<SurfaceFact> = exported
            .map(|ident| SurfaceFact::Export {
                crate_name: crate_name.clone(),
                name: ident.to_string(),
                path: Arc::clone(path),
                line: ident.span().start().line,
            })
            .collect();
        self.facts.extend(facts);
    }
}

impl Hooks for PublicSurface<'_> {
    fn file(&mut self, file: &syn::File) -> Descent {
        self.source.library_root().map_or_default(|()| self.exports(&file.items));
        Descent::from((self.reader != Reader::Neither).then_some(()))
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let sightings: Vec<SurfaceFact> = UseLeaf::of_tree(&item.tree)
            .iter()
            .flat_map(|leaf| {
                let first = leaf.path().first().copied();
                let reached = leaf.end().original().map_or(Reached::Glob, |name| Reached::Named(name.to_string()));
                let module =
                    leaf.path().get(1).and_then(|module| self.sighting(first, Reached::Named(module.to_string())));
                self.sighting(first, reached).into_iter().chain(module)
            })
            .collect();
        self.facts.extend(sightings);
        Descent::Over
    }

    fn path(&mut self, path: &syn::Path) -> Descent {
        let first = path.segments.first().map(|segment| &segment.ident);
        let second = path.segments.iter().nth(1);
        let sighting = second.and_then(|named| self.sighting(first, Reached::Named(named.ident.to_string())));
        self.facts.extend(sighting);
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Surface
    }
}

impl<'file> Graph<'file> for PublicSurface<'file> {
    type Fact = SurfaceFact;

    fn new(source: &'file Source) -> PublicSurface<'file> {
        let reader = match source.place() {
            Place::Shipped => Reader::Shipped,
            Place::Tests => Reader::Tests,
            Place::Elsewhere => Reader::Neither,
        };
        let own_crate = source.own_crate();
        PublicSurface { source, reader, own_crate, facts: Vec::new() }
    }

    fn facts(self) -> Vec<SurfaceFact> {
        let tested = (self.reader == Reader::Tests).then(|| self.source.package().crate_name());
        self.facts
            .into_iter()
            .filter(|fact| match (fact, &tested) {
                (SurfaceFact::Sighting { crate_name, .. }, Some(own)) => crate_name == own,
                _ => true,
            })
            .collect()
    }

    fn reduce(facts: Vec<SurfaceFact>, _: &[Arc<Package>]) -> Vec<Finding> {
        let reaches = facts.iter().fold(BTreeMap::new(), |mut reaches: BTreeMap<&str, Reach>, fact| {
            let sighted = match fact {
                SurfaceFact::Sighting { crate_name, reached, reader } => Some(crate_name).zip(Some((reached, *reader))),
                SurfaceFact::Export { .. } => None,
            };
            sighted.map_or_default(|(crate_name, (reached, reader))| {
                reaches.entry(crate_name.as_str()).or_default().absorb(reached, reader);
            });
            reaches
        });
        facts
            .iter()
            .filter_map(|fact| match fact {
                SurfaceFact::Export { crate_name, name, path, line } => {
                    let reach = reaches.get(crate_name.as_str());
                    let unnamed = reach.filter(|known| known.used == Used::Named && !known.names.contains(name));
                    unnamed.map(|_| {
                        Found::at(
                            *line,
                            format_args!("{name} is exported but no other crate names it"),
                            "export it when a user appears",
                        )
                        .in_file(Rule::PublicSurface, path)
                    })
                }
                SurfaceFact::Sighting { .. } => None,
            })
            .collect()
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use proc_macro2::Ident;
use syn::{Item, ItemUse, Visibility};

use super::Spot;
use crate::finding::{Finding, Found, Rule};
use crate::source::{Place, Source};
use crate::terms::{Fix, Line, Name};
use crate::use_leaf::UseLeaf;
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum SurfaceFact {
    Export { crate_name: Name, name: Name, path: Arc<Path>, line: Line },
    Sighting { crate_name: Name, reached: Reached, reader: Reader },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reached {
    Named(Name),
    Glob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Reader {
    Shipped,
    Tests,
    ForeignTests,
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
    names: BTreeSet<Name>,
    used: Used,
}

#[derive(Debug)]
pub(super) struct Surface<'file> {
    source: &'file Source,
    reader: Reader,
    own_crate: Option<Name>,
    facts: Vec<SurfaceFact>,
}

impl Reach {
    fn absorb(&mut self, reached: &Reached, reader: Reader) {
        let used = match (reached, reader) {
            (_, Reader::Tests | Reader::ForeignTests | Reader::Neither) => Used::Unused,
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

impl<'file> Surface<'file> {
    const FIX: Fix = Fix("export it when a user appears");

    pub(super) fn new(source: &'file Source) -> Surface<'file> {
        let reader = match source.place() {
            Place::Shipped => Reader::Shipped,
            Place::Tests => Reader::Tests,
            Place::Elsewhere => Reader::Neither,
        };
        Surface { source, reader, own_crate: source.own_crate(), facts: Vec::new() }
    }

    fn sighting(&self, first: Option<&Ident>, reached: Reached) -> Option<SurfaceFact> {
        let foreign = first.map(Name::of).filter(|named| Some(named) != self.own_crate.as_ref());
        foreign.map(|crate_name| SurfaceFact::Sighting { crate_name, reached, reader: self.reader })
    }

    pub(super) fn filed(&mut self, file: &syn::File) -> Option<()> {
        self.source.library_root().map_or_default(|()| self.exports(&file.items));
        (self.reader != Reader::Neither).then_some(())
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
                name: Name::of(ident),
                path: Arc::clone(path),
                line: Line::of(ident.span()),
            })
            .collect();
        self.facts.extend(facts);
    }

    pub(super) fn used(&mut self, item: &ItemUse) {
        let sightings: Vec<SurfaceFact> = UseLeaf::of_tree(&item.tree)
            .iter()
            .flat_map(|leaf| {
                let first = leaf.path().first().copied();
                let reached = leaf.end().original().map_or(Reached::Glob, |name| Reached::Named(Name::of(name)));
                let module =
                    leaf.path().get(1).and_then(|module| self.sighting(first, Reached::Named(Name::of(module))));
                self.sighting(first, reached).into_iter().chain(module)
            })
            .collect();
        self.facts.extend(sightings);
    }

    pub(super) fn pathed(&mut self, path: &syn::Path) {
        let first = path.segments.first().map(|segment| &segment.ident);
        let second = path.segments.iter().nth(1);
        let sighting = second.and_then(|named| self.sighting(first, Reached::Named(Name::of(&named.ident))));
        self.facts.extend(sighting);
    }

    pub(super) fn facts(self) -> Vec<SurfaceFact> {
        let tested = (self.reader == Reader::Tests).then(|| self.source.package().crate_name());
        self.facts
            .into_iter()
            .map(|fact| match (fact, &tested) {
                (SurfaceFact::Sighting { crate_name, reached, .. }, Some(own)) if crate_name != *own => {
                    SurfaceFact::Sighting { crate_name, reached, reader: Reader::ForeignTests }
                }
                (kept, _) => kept,
            })
            .collect()
    }

    pub(super) fn reduce(facts: &[SurfaceFact], packages: &[Arc<Package>], dead: &BTreeSet<Spot>) -> Vec<Finding> {
        let dev_only: BTreeSet<Name> =
            packages.iter().filter(|package| package.shipped().is_none()).map(|package| package.crate_name()).collect();
        let reaches = facts.iter().fold(BTreeMap::new(), |mut reaches: BTreeMap<&Name, Reach>, fact| {
            let sighted = match fact {
                SurfaceFact::Sighting { crate_name, reached, reader: Reader::ForeignTests } => {
                    Some(crate_name).filter(|named| dev_only.contains(*named)).zip(Some((reached, Reader::Shipped)))
                }
                SurfaceFact::Sighting { crate_name, reached, reader } => Some(crate_name).zip(Some((reached, *reader))),
                SurfaceFact::Export { .. } => None,
            };
            sighted.map_or_default(|(crate_name, (reached, reader))| {
                reaches.entry(crate_name).or_default().absorb(reached, reader);
            });
            reaches
        });
        facts
            .iter()
            .filter_map(|fact| match fact {
                SurfaceFact::Export { crate_name, name, path, line } => {
                    let reach = reaches.get(crate_name);
                    let unnamed = reach.filter(|known| known.used == Used::Named && !known.names.contains(name));
                    let fresh = unnamed.filter(|_| !dead.contains(&Spot::new(Arc::clone(path), *line, name.clone())));
                    fresh.map(|_| {
                        Found::at(*line, format_args!("{name} is exported but no other crate names it"), Surface::FIX)
                            .in_file(Rule::DeadCode, path)
                    })
                }
                SurfaceFact::Sighting { .. } => None,
            })
            .collect()
    }
}

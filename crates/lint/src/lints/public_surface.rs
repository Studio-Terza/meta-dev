use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use pipeline::{Edges, Measure};
use proc_macro2::Ident;
use syn::{Item, ItemUse, Macro, Visibility};

use crate::finding::{Finding, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Fact, Lint, Reads};
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

#[derive(Debug, Default, Measure)]
pub(crate) struct SurfaceFacts(Vec<SurfaceFact>);

#[derive(Debug)]
pub(crate) struct PublicSurface<'file> {
    source: &'file Source,
    reader: Reader,
    own_crate: Option<String>,
    facts: Vec<SurfaceFact>,
}

impl Used {
    fn of(reached: &Reached, reader: Reader) -> Used {
        match (reached, reader) {
            (_, Reader::Tests | Reader::Neither) => Used::Unused,
            (Reached::Named(_), Reader::Shipped) => Used::Named,
            (Reached::Glob, Reader::Shipped) => Used::Globbed,
        }
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

impl<'file> Lint<'file> for PublicSurface<'file> {
    const HEADLINE: &'static str = "a crate exports exactly what another crate uses";
    const READS: Reads = Reads::Every;

    type Fact = SurfaceFacts;

    fn new(source: &'file Source) -> PublicSurface<'file> {
        let reader = match source.place() {
            Place::Shipped => Reader::Shipped,
            Place::Tests => Reader::Tests,
            Place::Elsewhere => Reader::Neither,
        };
        let own_crate = source.own_crate();
        PublicSurface { source, reader, own_crate, facts: Vec::new() }
    }

    fn fact(self, _: &'file Source) -> SurfaceFacts {
        let tested = (self.reader == Reader::Tests).then(|| self.source.package().crate_name());
        let kept = self.facts.into_iter().filter(|fact| match (fact, &tested) {
            (SurfaceFact::Sighting { crate_name, .. }, Some(own)) => crate_name == own,
            _ => true,
        });
        SurfaceFacts(kept.collect())
    }
}

impl Fact for SurfaceFacts {
    fn findings(self, _: &[Arc<Package>]) -> Vec<Finding> {
        let sightings: Vec<(&str, &Reached, Reader)> = self
            .0
            .iter()
            .filter_map(|fact| match fact {
                SurfaceFact::Sighting { crate_name, reached, reader } => Some((crate_name.as_str(), reached, *reader)),
                SurfaceFact::Export { .. } => None,
            })
            .collect();
        let named: Edges<&str, &str> = Edges::total(sightings.iter().map(|(crate_name, reached, _)| match reached {
            Reached::Named(name) => Edges::edge(*crate_name, name.as_str()),
            Reached::Glob => Edges::node(*crate_name),
        }));
        let used =
            sightings.iter().fold(BTreeMap::new(), |mut used: BTreeMap<&str, Used>, (crate_name, reached, reader)| {
                let entry = used.entry(crate_name).or_default();
                *entry = (*entry).max(Used::of(reached, *reader));
                used
            });
        self.0
            .iter()
            .filter_map(|fact| match fact {
                SurfaceFact::Export { crate_name, name, path, line } => {
                    let named_only = used.get(crate_name.as_str()).filter(|reach| **reach == Used::Named);
                    let unnamed = named_only
                        .and_then(|_| named.targets(&crate_name.as_str()))
                        .filter(|names| !names.contains(name.as_str()));
                    unnamed.map(|_| {
                        Found::at(
                            *line,
                            format_args!("{name} is exported but no other crate names it"),
                            "export it when a user appears",
                        )
                        .in_file(PublicSurface::HEADLINE, path)
                    })
                }
                SurfaceFact::Sighting { .. } => None,
            })
            .collect()
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use proc_macro2::Ident;
use syn::{ItemUse, Macro, VisRestricted};

use crate::finding::{Finding, Found, Rule};
use crate::graph::Graph;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::source::{Place, Source};
use crate::terms::{Fix, Name};
use crate::use_leaf::UseLeaf;
use crate::workspace::{Needed, Package};

#[derive(Debug)]
pub(crate) struct ForeignFact {
    foreign: Name,
    package: Name,
}

#[derive(Debug)]
pub(crate) struct PortModel<'file> {
    source: &'file Source,
    foreign: BTreeSet<Name>,
    named: BTreeSet<Name>,
}

impl PortModel<'_> {
    const FIX: Fix = Fix("confine it to one port crate that wraps it in our own types, and depend on that crate");

    fn sighted<'ident>(&mut self, firsts: impl IntoIterator<Item = &'ident Ident>) {
        let named: Vec<Name> = firsts.into_iter().map(Name::of).filter(|name| self.foreign.contains(name)).collect();
        self.named.extend(named);
    }
}

impl Hooks for PortModel<'_> {
    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let leaves = UseLeaf::of_tree(&item.tree);
        self.sighted(leaves.iter().filter_map(|leaf| leaf.path().first().copied().or_else(|| leaf.end().original())));
        Descent::Over
    }

    fn path(&mut self, path: &syn::Path) -> Descent {
        let qualified = path.segments.iter().nth(1).and(path.segments.first());
        self.sighted(qualified.map(|segment| &segment.ident));
        Descent::Into
    }

    fn vis_restricted(&mut self, _: &VisRestricted) -> Descent {
        Descent::Over
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Surface
    }
}

impl<'file> Graph<'file> for PortModel<'file> {
    type Fact = ForeignFact;

    fn new(source: &'file Source) -> PortModel<'file> {
        let shipped = (source.place() == Place::Shipped).then_some(());
        let foreign = shipped.map_or_default(|()| {
            let dependencies = source.package().dependencies().iter();
            let needed = dependencies.filter(|dependency| dependency.needed() == Needed::Shipped);
            needed.map(|dependency| dependency.name().clone()).collect()
        });
        PortModel { source, foreign, named: BTreeSet::new() }
    }

    fn facts(self) -> Vec<ForeignFact> {
        let package = self.source.package().name();
        self.named.into_iter().map(|foreign| ForeignFact { foreign, package: package.clone() }).collect()
    }

    fn reduce(facts: Vec<ForeignFact>, packages: &[Arc<Package>]) -> Vec<Finding> {
        let own: BTreeSet<Name> = packages.iter().map(|package| package.crate_name()).collect();
        let by_name: BTreeMap<&Name, &Arc<Package>> =
            packages.iter().map(|package| (package.name(), package)).collect();
        let namers = facts.into_iter().filter(|fact| !own.contains(&fact.foreign)).fold(
            BTreeMap::new(),
            |mut namers: BTreeMap<Name, BTreeSet<Name>>, fact| {
                namers.entry(fact.foreign).or_default().insert(fact.package);
                namers
            },
        );
        namers
            .iter()
            .filter(|(_, named_by)| named_by.len() > 1)
            .flat_map(|(foreign, named_by)| {
                let listed = named_by.iter().map(Name::text).collect::<Vec<&str>>().join(", ");
                named_by.iter().filter_map(|name| by_name.get(name)).filter_map(move |package| {
                    let line = package
                        .dependencies()
                        .iter()
                        .find(|dependency| dependency.name() == foreign && dependency.needed() == Needed::Shipped);
                    line.map(|dependency| {
                        let problem = format_args!("{foreign} is a foreign interface named in the code of {listed}");
                        Found::at(dependency.line(), problem, PortModel::FIX)
                            .in_file(Rule::PortModel, package.manifest())
                    })
                })
            })
            .collect()
    }
}

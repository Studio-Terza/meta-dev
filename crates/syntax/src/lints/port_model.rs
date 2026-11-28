use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use pipeline::Measure;
use proc_macro2::Ident;
use syn::{ItemUse, Macro, VisRestricted, Visibility};

use crate::finding::{Finding, Findings, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Fact, Lint, Reads};
use crate::source::{Place, Source};
use crate::use_leaf::UseLeaf;
use crate::workspace::{Needed, Package};

#[derive(Debug)]
pub(crate) struct ForeignFact {
    foreign: String,
    package: String,
}

pipeline::measured! {
    #[derive(Debug)]
    pub(crate) struct ForeignFacts {
        named: Vec<ForeignFact>,
        leaked: Findings,
    }
}

#[derive(Debug)]
pub(crate) struct PortModel<'file> {
    source: &'file Source,
    foreign: BTreeSet<String>,
    named: BTreeSet<String>,
    leaked: Vec<Found>,
}

impl PortModel<'_> {
    const FIX: &'static str = "confine it to one port crate that wraps it in our own types, and depend on that crate";
    const LEAK_FIX: &'static str = "wrap what callers need in the port's own types instead of re-exporting the library";

    fn foreign(&self, first: &Ident) -> Option<String> {
        Some(first.to_string()).filter(|name| self.foreign.contains(name))
    }
}

impl Hooks for PortModel<'_> {
    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let leaves = UseLeaf::of_tree(&item.tree);
        let firsts: Vec<String> = leaves
            .iter()
            .filter_map(|leaf| leaf.path().first().copied().or_else(|| leaf.end().original()))
            .filter_map(|first| self.foreign(first))
            .collect();
        let public = match item.vis {
            Visibility::Public(_) => firsts.first().map(|_| ()),
            Visibility::Restricted(_) | Visibility::Inherited => None,
        };
        self.leaked.extend(public.map(|()| {
            Found::spanned(item.use_token.span, format_args!("pub use of {}", firsts.join(", ")), PortModel::LEAK_FIX)
        }));
        self.named.extend(firsts);
        Descent::Over
    }

    fn path(&mut self, path: &syn::Path) -> Descent {
        let qualified = path.segments.iter().nth(1).and(path.segments.first());
        self.named.extend(qualified.and_then(|segment| self.foreign(&segment.ident)));
        Descent::Into
    }

    fn vis_restricted(&mut self, _: &VisRestricted) -> Descent {
        Descent::Over
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Path
    }
}

impl<'file> Lint<'file> for PortModel<'file> {
    const HEADLINE: &'static str = "a foreign crate named in code lives in exactly one port crate";
    const READS: Reads = Reads::Shipped;

    type Fact = ForeignFacts;

    fn new(source: &'file Source) -> PortModel<'file> {
        let shipped = (source.place() == Place::Shipped).then_some(());
        let foreign = shipped.map_or_default(|()| {
            let dependencies = source.package().dependencies().iter();
            let needed = dependencies.filter(|dependency| dependency.needed() == Needed::Shipped);
            needed.map(|dependency| dependency.name().to_owned()).collect()
        });
        PortModel { source, foreign, named: BTreeSet::new(), leaked: Vec::new() }
    }

    fn fact(self, source: &'file Source) -> ForeignFacts {
        let package = source.package().crate_name();
        let named = self.named.into_iter().map(|foreign| ForeignFact { foreign, package: package.clone() }).collect();
        ForeignFacts { named, leaked: Findings::found(PortModel::HEADLINE, self.source.shown(), self.leaked) }
    }
}

impl Fact for ForeignFacts {
    fn findings(self, packages: &[Arc<Package>]) -> Vec<Finding> {
        let by_name: BTreeMap<String, &Arc<Package>> =
            packages.iter().map(|package| (package.crate_name(), package)).collect();
        let namers: BTreeMap<String, BTreeSet<String>> = BTreeMap::total(
            self.named.into_iter().map(|fact| BTreeMap::from([(fact.foreign, BTreeSet::from([fact.package]))])),
        );
        let shared = namers.into_iter().filter(|(_, named_by)| named_by.len() > 1).flat_map(|(foreign, named_by)| {
            let listed = named_by.iter().map(String::as_str).collect::<Vec<&str>>().join(", ");
            let declaring = named_by.iter().filter_map(|name| by_name.get(name));
            declaring
                .filter_map(|package| {
                    let declared = package
                        .dependencies()
                        .iter()
                        .find(|dependency| dependency.name() == foreign && dependency.needed() == Needed::Shipped);
                    declared.map(|dependency| {
                        let problem = format!("{foreign} is a foreign interface named in the code of {listed}");
                        Found::at(dependency.line(), problem, PortModel::FIX)
                            .in_file(PortModel::HEADLINE, package.manifest())
                    })
                })
                .collect::<Vec<Finding>>()
        });
        self.leaked.into_iter().chain(shared).collect()
    }
}

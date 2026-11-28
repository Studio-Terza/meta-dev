use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter;
use std::path::Path;
use std::sync::Arc;

use proc_macro2::Ident;
use syn::{ItemMod, ItemUse, Macro, VisRestricted};

use crate::finding::{Finding, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Fact, Lint, Reads};
use crate::source::Source;
use crate::use_leaf::UseLeaf;
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum ModuleFact {
    Declared { root: Arc<Path>, module: Vec<String> },
    Named(Naming),
}

pipeline::measured! {
    #[derive(Debug)]
    pub(crate) struct ModuleFacts(Vec<ModuleFact>);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Anchor {
    Crate,
    Below,
}

#[derive(Clone, Debug)]
pub(crate) struct Naming {
    root: Arc<Path>,
    from: Vec<String>,
    named: Vec<String>,
    anchor: Anchor,
    binds: Option<String>,
    spot: Spot,
}

#[derive(Clone, Debug)]
struct Spot {
    path: Arc<Path>,
    line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    from: Vec<String>,
    to: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Binding {
    module: Vec<String>,
    name: String,
}

#[derive(Debug, Default)]
struct Crate {
    modules: BTreeSet<Vec<String>>,
    bindings: BTreeMap<Binding, Naming>,
    namings: Vec<Naming>,
}

#[derive(Debug, Default)]
struct Edges(BTreeMap<Edge, Spot>);

#[derive(Debug)]
struct Shown<'module>(&'module [String]);

#[derive(Debug)]
pub(crate) struct ModuleCycles<'file> {
    source: &'file Source,
    open: Vec<Vec<String>>,
    facts: Vec<ModuleFact>,
}

impl ModuleCycles<'_> {
    const CRATE: &'static str = "crate";
    const PARENT: &'static str = "super";
    const OWN: &'static str = "self";
    const FIX: &'static str =
        "move what they share into a module that names neither, or split the crate along the cycle";

    fn current(&self) -> Vec<String> {
        self.open.last().cloned().unwrap_or_default()
    }

    fn naming(&self, segments: &[String], binds: Option<String>, line: usize) -> Option<ModuleFact> {
        let from = self.current();
        let resolved = match segments.split_first() {
            Some((first, rest)) if first == ModuleCycles::CRATE => Some((rest.to_vec(), Anchor::Crate)),
            Some((first, rest)) if first == ModuleCycles::OWN => {
                Some((from.iter().chain(rest).cloned().collect(), Anchor::Crate))
            }
            Some((first, _)) if first == ModuleCycles::PARENT => {
                let climbed = segments.iter().take_while(|segment| *segment == ModuleCycles::PARENT).count();
                from.len().checked_sub(climbed).map(|kept| {
                    let base = from.iter().take(kept);
                    (base.chain(segments.iter().skip(climbed)).cloned().collect(), Anchor::Crate)
                })
            }
            Some(_) => Some((from.iter().chain(segments).cloned().collect(), Anchor::Below)),
            None => None,
        };
        resolved.map(|(named, anchor)| {
            ModuleFact::Named(Naming {
                root: Arc::clone(self.source.module().crate_root()),
                from: from.clone(),
                named,
                anchor,
                binds,
                spot: Spot { path: Arc::clone(self.source.shown()), line },
            })
        })
    }

    fn spelled<'ident>(idents: impl IntoIterator<Item = &'ident Ident>) -> Vec<String> {
        idents.into_iter().map(ToString::to_string).collect()
    }
}

impl Naming {
    fn target(&self, modules: &BTreeSet<Vec<String>>) -> Option<Vec<String>> {
        let floor = match self.anchor {
            Anchor::Crate => 0,
            Anchor::Below => self.from.len() + 1,
        };
        (floor..=self.named.len())
            .rev()
            .filter_map(|length| self.named.get(..length))
            .find(|prefix| modules.contains(*prefix))
            .map(<[String]>::to_vec)
    }

    fn item(&self, module: &[String]) -> Option<Binding> {
        self.named.get(module.len()).map(|name| Binding { module: module.to_vec(), name: name.clone() })
    }
}

impl Crate {
    fn absorb(&mut self, naming: Naming) {
        let bound = naming.binds.clone().map(|name| Binding { module: naming.from.clone(), name });
        bound.map_or_default(|binding| {
            self.bindings.insert(binding, naming.clone());
        });
        self.namings.push(naming);
    }

    fn resolved(&self, naming: &Naming) -> Option<Vec<String>> {
        let start = naming.target(&self.modules).map(|target| (naming, target));
        let steps = iter::successors(Some(start), |step| {
            let (current, target) = step.as_ref()?;
            let alias = current.item(target).and_then(|item| self.bindings.get(&item))?;
            Some(alias.target(&self.modules).map(|next| (alias, next)))
        });
        steps.take(self.bindings.len() + 1).last().flatten().map(|(_, target)| target)
    }

    fn edges(&self) -> Edges {
        let mut edges = Edges::default();
        self.namings.iter().for_each(|naming| {
            let to = self.resolved(naming).filter(|target| *target != naming.from);
            to.map_or_default(|target| {
                edges.0.entry(Edge { from: naming.from.clone(), to: target }).or_insert_with(|| naming.spot.clone());
            });
        });
        edges
    }

    fn found(&self) -> Vec<Finding> {
        let edges = self.edges();
        let reaches: BTreeMap<&Vec<String>, BTreeSet<Vec<String>>> =
            self.modules.iter().map(|module| (module, edges.reach(module))).collect();
        let cycles: BTreeSet<BTreeSet<Vec<String>>> = reaches
            .iter()
            .filter(|(module, reached)| reached.contains(**module))
            .map(|(module, reached)| {
                let back = |other: &&Vec<String>| reaches.get(*other).is_some_and(|known| known.contains(*module));
                reached.iter().filter(back).cloned().collect()
            })
            .collect();
        cycles.iter().filter_map(|cycle| edges.cycle(cycle)).collect()
    }
}

impl Edges {
    fn reach(&self, start: &[String]) -> BTreeSet<Vec<String>> {
        let first = self.next(iter::once(start));
        let grown = iter::successors(Some(first), |known| {
            let reached: BTreeSet<Vec<String>> =
                self.next(known.iter().map(Vec::as_slice)).difference(known).cloned().collect();
            reached.first().map(|_| known.union(&reached).cloned().collect())
        });
        grown.last().unwrap_or_default()
    }

    fn next<'module>(&self, from: impl Iterator<Item = &'module [String]>) -> BTreeSet<Vec<String>> {
        let sources: BTreeSet<&[String]> = from.collect();
        self.0.keys().filter(|edge| sources.contains(edge.from.as_slice())).map(|edge| edge.to.clone()).collect()
    }

    fn cycle(&self, cycle: &BTreeSet<Vec<String>>) -> Option<Finding> {
        let inside: Vec<(&Edge, &Spot)> =
            self.0.iter().filter(|(edge, _)| cycle.contains(&edge.from) && cycle.contains(&edge.to)).collect();
        let modules: Vec<String> = cycle.iter().map(|module| Shown(module).to_string()).collect();
        let uses: Vec<String> = inside
            .iter()
            .map(|(edge, spot)| {
                format!("{} names {} at {}:{}", Shown(&edge.from), Shown(&edge.to), spot.path.display(), spot.line)
            })
            .collect();
        inside.first().map(|(_, spot)| {
            let problem = format_args!("modules {} import each other: {}", modules.join(", "), uses.join(", "));
            Found::at(spot.line, problem, ModuleCycles::FIX).in_file(ModuleCycles::HEADLINE, &spot.path)
        })
    }
}

impl fmt::Display for Shown<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(ModuleCycles::CRATE)?;
        self.0.iter().try_for_each(|name| write!(formatter, "::{name}"))
    }
}

impl Hooks for ModuleCycles<'_> {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        let inline = module.content.as_ref().map(|_| {
            let child: Vec<String> = self.current().into_iter().chain([module.ident.to_string()]).collect();
            self.facts.push(ModuleFact::Declared {
                root: Arc::clone(self.source.module().crate_root()),
                module: child.clone(),
            });
            child
        });
        self.open.push(inline.unwrap_or_else(|| self.current()));
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.open.pop();
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let rooted = item.leading_colon.is_none().then_some(());
        let leaves = rooted.map_or_default(|()| UseLeaf::of_tree(&item.tree));
        let namings: Vec<ModuleFact> = leaves
            .iter()
            .filter_map(|leaf| {
                let end = leaf.end().original().filter(|ident| *ident != ModuleCycles::OWN);
                let last = end.or_else(|| leaf.path().last().copied());
                let own = leaf.end().original().filter(|ident| *ident == ModuleCycles::OWN);
                let renamed = leaf.end().bound().filter(|ident| *ident != ModuleCycles::OWN);
                let bound = renamed.or(own.and(leaf.path().last().copied()));
                let line = last.map_or(item.use_token.span.start().line, |ident| ident.span().start().line);
                let segments = ModuleCycles::spelled(leaf.path().iter().copied().chain(end));
                self.naming(&segments, bound.map(ToString::to_string), line)
            })
            .collect();
        self.facts.extend(namings);
        Descent::Over
    }

    fn path(&mut self, path: &syn::Path) -> Descent {
        let segments = ModuleCycles::spelled(path.segments.iter().map(|segment| &segment.ident));
        let qualified = (path.leading_colon.is_none() && segments.len() > 1).then_some(());
        let line = path.segments.first().map_or(0, |segment| segment.ident.span().start().line);
        let naming = qualified.and_then(|()| self.naming(&segments, None, line));
        self.facts.extend(naming);
        Descent::Into
    }

    fn vis_restricted(&mut self, _: &VisRestricted) -> Descent {
        Descent::Over
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Surface
    }
}

impl<'file> Lint<'file> for ModuleCycles<'file> {
    const HEADLINE: &'static str = "modules inside a crate never import each other in a cycle";
    const READS: Reads = Reads::All;

    type Fact = ModuleFacts;

    fn new(source: &'file Source) -> ModuleCycles<'file> {
        let module = source.module();
        let declared = ModuleFact::Declared { root: Arc::clone(module.crate_root()), module: module.names().to_vec() };
        ModuleCycles { source, open: vec![module.names().to_vec()], facts: vec![declared] }
    }

    fn fact(self, _: &'file Source) -> ModuleFacts {
        ModuleFacts(self.facts)
    }
}

impl Fact for ModuleFacts {
    fn findings(self, _: &[Arc<Package>]) -> Vec<Finding> {
        let mut crates: BTreeMap<Arc<Path>, Crate> = BTreeMap::new();
        let (declared, named): (Vec<ModuleFact>, Vec<ModuleFact>) =
            self.0.into_iter().partition(|fact| matches!(fact, ModuleFact::Declared { .. }));
        declared.into_iter().chain(named).for_each(|fact| match fact {
            ModuleFact::Declared { root, module } => {
                crates.entry(root).or_default().modules.insert(module);
            }
            ModuleFact::Named(naming) => crates.get_mut(&naming.root).map_or_default(|known| known.absorb(naming)),
        });
        crates.values().flat_map(Crate::found).collect()
    }
}

mod count;
mod definition;
mod link;
mod tally;
mod tangle;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_item_graph::ItemGraph;
use type_facts::{Facts, FactsLint};

use definition::{Declared, Definition};
use link::Link;
use tally::Tally;
use tangle::Tangle;

#[derive(Debug)]
pub struct Layering;

impl Layering {
    fn violations<'file>(declared: &Declared<'_, 'file>, links: &[Link<'_, 'file>]) -> Vec<Violation<'file>> {
        let graph =
            ItemGraph::of_references(links.iter().fold(BTreeMap::new(), |references, link| link.added_to(references)));
        let cycles = graph
            .tangles()
            .into_iter()
            .filter(|component| component.len() > 1)
            .map(|component| component.into_iter().filter_map(|index| graph.name_at(index).copied()).collect());
        let loops: BTreeSet<Definition<'_>> = links.iter().filter_map(Link::direct_loop).collect();
        let tangles = Tangle::ranked(
            cycles
                .chain(loops.into_iter().map(|node| BTreeSet::from([node])))
                .map(|members| Tangle::closed(members, links, declared))
                .collect(),
        );
        let tally = Tally::counted(declared.count(), links, &tangles);
        declared
            .first()
            .map(|site| Violation::new(site, tally))
            .into_iter()
            .chain(tangles.iter().map(Violation::from))
            .collect()
    }
}

impl FactsLint for Layering {
    fn report_facts<'file>(facts: &Facts<'file>) -> Report<'file> {
        let declared = Declared::of_facts(facts);
        let links: Vec<Link<'_, 'file>> = declared.links(facts);
        Report::new(
            Headline::new(
                "report only, layering: strongly connected groups of type and trait definitions over the definition graph, where A depends on B when A has a field of type B, an enum variant carrying B, or a bound on B, resolved by name within the workspace (same crate first, then a crate-qualified path, then the one crate declaring it); self-recursion through a reference, Box, Rc, Arc, Weak, Vec or VecDeque is allowed, every other cycle is a tangle",
            ),
            Self::violations(&declared, &links),
        )
    }
}

impl Lint for Layering {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::report_facts(&Facts::of_files(files))
    }
}

mod crate_graph;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_placement::CrateName;
use type_facts::{Facts, TraitFact, TypeFact};

use crate_graph::CrateGraph;

#[derive(Debug)]
pub struct TypeGraph;

impl TypeGraph {
    fn grouped<'facts>(
        types: &'facts [TypeFact],
        traits: &'facts [TraitFact],
    ) -> BTreeMap<&'facts CrateName, CrateGraph<'facts>> {
        let with_types =
            types.iter().fold(BTreeMap::new(), |mut crates: BTreeMap<&CrateName, CrateGraph<'_>>, found| {
                crates.entry(found.as_ref()).or_default().add_type(found);
                crates
            });
        traits.iter().fold(with_types, |mut crates, found| {
            crates.entry(found.as_ref()).or_default().add_trait(found);
            crates
        })
    }
}

impl Lint for TypeGraph {
    fn report(files: &[SourceFile]) -> Report {
        let facts = Facts::of_files(files);
        Report::new(
            Headline::new(
                "report only, the type-trait graph per crate, types and local traits as nodes, conversions (From, TryFrom, AsRef, Into) and local trait impls as edges; resolved by name, so same-named types merge",
            ),
            Self::grouped(facts.types(), facts.traits())
                .into_iter()
                .filter_map(|(name, graph)| graph.violation(name))
                .collect(),
        )
    }
}

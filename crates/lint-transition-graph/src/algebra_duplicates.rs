use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Fix, Headline, Report, Site, Violation};
use lint_placement::CrateName;

use crate::algebra::Twins;
use crate::algebra_laws::attached::Attached;
use crate::count::Count;
use crate::graph_lint::GraphLint;
use crate::listed::Listed;
use crate::system::TypeSystem;

#[derive(Debug)]
pub struct AlgebraDuplicates;

impl GraphLint for AlgebraDuplicates {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        let algebras = system.algebras();
        let attached = Attached::declared(system);
        let twins: Vec<Twins<'_, '_, 'file>> = algebras
            .algebras()
            .flat_map(|algebra| algebra.twins(|operation| attached.of_operation(operation)))
            .collect();
        let by_crate = twins.iter().fold(
            BTreeMap::new(),
            |mut crates: BTreeMap<&CrateName, Vec<&Twins<'_, '_, 'file>>>, group| {
                crates.entry(AsRef::<CrateName>::as_ref(group.subject())).or_default().push(group);
                crates
            },
        );
        Report::new(
            Headline::new(
                "report only, algebra duplicates: operations of one type with the same signature (value inputs, external parameters, total or partial) that declare the same laws, two or more, such as join and merge both associative; only operations that declare a law are compared, the rest belong to algebra_laws; listed per crate",
            ),
            by_crate.iter().filter_map(|(name, groups)| CrateTwins::measured(name, groups).reported()).collect(),
        )
    }
}

#[derive(Debug)]
struct CrateTwins<'op, 'found, 'file> {
    name: &'op CrateName,
    first: Option<Site<'file>>,
    groups: Vec<&'op Twins<'op, 'found, 'file>>,
    members: Count,
}

impl<'op, 'found, 'file> CrateTwins<'op, 'found, 'file> {
    fn measured(name: &'op CrateName, groups: &[&'op Twins<'op, 'found, 'file>]) -> CrateTwins<'op, 'found, 'file> {
        CrateTwins {
            name,
            first: groups
                .iter()
                .flat_map(|group| group.members())
                .map(|member| *AsRef::<Site<'file>>::as_ref(*member))
                .min(),
            groups: groups.to_vec(),
            members: Count::new(groups.iter().map(|group| group.members().len()).sum()),
        }
    }

    fn reported(self) -> Option<Violation<'file>> {
        self.first.map(|site| Violation::new(site, &self))
    }
}

impl fmt::Display for CrateTwins<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let groups = Listed::<_, ';'>::joined(self.groups.iter());
        write!(
            formatter,
            "crate {}: {} operations in {} groups share a signature and laws; {groups}",
            self.name,
            self.members,
            Count::new(self.groups.len()),
        )
    }
}

impl lint_core::Finding for CrateTwins<'_, '_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("keep one operation of each group, or give the others a different signature or a law of their own")
    }
}

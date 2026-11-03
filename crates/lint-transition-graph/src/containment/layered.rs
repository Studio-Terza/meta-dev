use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_core::{Fix, Headline, Report, Violation};
use lint_item_graph::ItemGraph;

use super::cluster::{Cluster, Part};
use super::member::Member;
use super::{ClusterLint, Containment};

#[derive(Debug)]
pub struct AcyclicQuotients;

impl ClusterLint for AcyclicQuotients {
    fn report_clusters<'file>(view: &Containment<'_, 'file>) -> Report<'file> {
        Report::new(
            Headline::new(
                "report only, the parts of each cluster are layered: collapse the crates of the workspace, and the own file and child modules of each crate and module, into single nodes of the type and trait graph, and no group of them uses one another in a cycle",
            ),
            view.clusters()
                .into_iter()
                .flat_map(|cluster| Cycle::all(view, cluster))
                .filter_map(|cycle| view.first_site(&cycle.members).map(|site| Violation::new(site, &cycle)))
                .collect(),
        )
    }
}

#[derive(Debug)]
struct Cycle<'found> {
    cluster: Cluster<'found>,
    parts: BTreeSet<Part<'found>>,
    members: BTreeSet<Member<'found>>,
    closing: Vec<(Member<'found>, Member<'found>)>,
}

impl<'found> Cycle<'found> {
    fn all(view: &Containment<'found, '_>, cluster: Cluster<'found>) -> Vec<Cycle<'found>> {
        let members = view.members(cluster);
        let part = |member: Member<'found>| Part::of(cluster, member.cluster());
        let crossing: Vec<(Member<'found>, Member<'found>)> = members
            .iter()
            .flat_map(|member| view.uses_of(*member).map(|used| (*member, used)).collect::<Vec<_>>())
            .filter(|(user, used)| members.contains(used) && part(*user) != part(*used))
            .collect();
        let quotient = members.iter().map(|member| (part(*member), BTreeSet::new())).chain(
            crossing.iter().map(|(user, used)| (part(*user), BTreeSet::from([part(*used)]))),
        );
        let references = quotient.fold(
            BTreeMap::new(),
            |mut references: BTreeMap<Part<'found>, BTreeSet<Part<'found>>>, (from, to)| {
                references.entry(from).or_default().extend(to);
                references
            },
        );
        let graph = ItemGraph::of_references(references);
        graph
            .tangles()
            .into_iter()
            .filter(|tangle| tangle.len() > 1)
            .map(|tangle| {
                let parts: BTreeSet<Part<'found>> =
                    tangle.into_iter().filter_map(|index| graph.name_at(index).copied()).collect();
                let closing: Vec<(Member<'found>, Member<'found>)> = crossing
                    .iter()
                    .copied()
                    .filter(|(user, used)| parts.contains(&part(*user)) && parts.contains(&part(*used)))
                    .collect();
                Cycle {
                    cluster,
                    members: closing.iter().flat_map(|(user, used)| [*user, *used]).collect(),
                    parts,
                    closing,
                }
            })
            .collect()
    }
}

impl fmt::Display for Cycle<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.parts.iter().map(ToString::to_string).collect();
        let closing: Vec<String> = self.closing.iter().map(|(user, used)| format!("{user} -> {used}")).collect();
        write!(
            formatter,
            "in {}, the parts {} use one another in a cycle, through {}",
            self.cluster,
            parts.join(", "),
            closing.join(", ")
        )
    }
}

impl lint_core::Finding for Cycle<'_> {
    fn fix(&self) -> Fix {
        Fix::new("move the shared types into one part that the others use, so the parts form layers")
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_core::{Fix, Headline, Violation};
use lint_item_graph::ItemGraph;

use super::cluster::{Cluster, Part};
use super::link::Link;
use super::member::Member;
use super::{ClusterLint, Containment, Toward};

#[derive(Debug)]
pub struct AcyclicQuotients;

impl ClusterLint for AcyclicQuotients {
    const HEADLINE: Headline = Headline::new(
        "report only, the parts of each cluster are layered: collapse the crates of the workspace, and the own file and child modules of each crate and module, into single nodes of the definition graph that layering reads (fields, variants and bounds), and no group of them depends on one another in a cycle; signatures and bodies may cycle, as functions do",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.clusters()
            .into_iter()
            .flat_map(|cluster| Cycle::all(view, cluster))
            .filter_map(|cycle| view.first_site(&cycle.members).map(|site| Violation::new(site, &cycle)))
            .collect()
    }
}

#[derive(Debug)]
struct Cycle<'found> {
    cluster: Cluster<'found>,
    parts: BTreeSet<Part<'found>>,
    members: BTreeSet<Member<'found>>,
    closing: Vec<Link<'found>>,
}

impl<'found> Cycle<'found> {
    fn all(view: &Containment<'found, '_>, cluster: Cluster<'found>) -> Vec<Cycle<'found>> {
        let members = view.members(cluster);
        let part = |member: Member<'found>| Part::toward(cluster, member.cluster());
        let parts = |link: &Link<'found>| link.ends().map(part);
        let crossing: Vec<Link<'found>> = view
            .links_within(&members, Toward::Defined)
            .into_iter()
            .filter(|link| {
                let [from, to] = parts(link);
                from != to
            })
            .collect();
        let quotient =
            members.iter().map(|member| (part(*member), BTreeSet::new())).chain(crossing.iter().map(|link| {
                let [from, to] = parts(link);
                (from, BTreeSet::from([to]))
            }));
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
            .map(|tangle| {
                let parts: BTreeSet<Part<'found>> =
                    tangle.into_iter().filter_map(|index| graph.name_at(index).copied()).collect();
                Cycle::closed(cluster, parts, &crossing)
            })
            .collect()
    }
}

impl<'found> Cycle<'found> {
    fn closed(cluster: Cluster<'found>, parts: BTreeSet<Part<'found>>, crossing: &[Link<'found>]) -> Cycle<'found> {
        let inside = |end: &Member<'found>| parts.contains(&Part::toward(cluster, end.cluster()));
        let closing: Vec<Link<'found>> =
            crossing.iter().copied().filter(|link| link.ends().iter().all(inside)).collect();
        Cycle { cluster, members: closing.iter().flat_map(Link::ends).collect(), parts, closing }
    }
}

impl fmt::Display for Cycle<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.parts.iter().map(ToString::to_string).collect();
        let closing: Vec<String> = self.closing.iter().map(ToString::to_string).collect();
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

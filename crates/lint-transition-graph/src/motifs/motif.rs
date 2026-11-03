use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::{Fix, Name};
use newtype::Counter;

use crate::count::Count;
use crate::end::{Hop, Node, Tuple};
use crate::graph::Hyperedge;

#[derive(Clone, Debug)]
pub(super) struct Occurrence<'graph, 'found> {
    edges: Vec<&'graph Hyperedge<'found>>,
    start: &'graph Tuple,
    reached: Vec<&'graph Node>,
}

impl<'graph, 'found> Occurrence<'graph, 'found> {
    pub(super) fn started(edge: &'graph Hyperedge<'found>) -> Occurrence<'graph, 'found> {
        let hop: &'graph Hop = edge.as_ref();
        Occurrence { edges: vec![edge], start: hop.as_ref(), reached: vec![hop.reached()] }
    }

    pub(super) fn extended(&self, edge: &'graph Hyperedge<'found>) -> Option<Occurrence<'graph, 'found>> {
        let hop: &'graph Hop = edge.as_ref();
        let available: BTreeSet<&'graph Node> = self.nodes().collect();
        hop.fired(&available).filter(|reached| !available.contains(reached)).map(|reached| Occurrence {
            edges: self.edges.iter().copied().chain([edge]).collect(),
            start: self.start,
            reached: self.reached.iter().copied().chain([reached]).collect(),
        })
    }

    pub(super) fn last(&self) -> Option<&'graph Node> {
        self.reached.last().copied()
    }

    pub(super) fn shape(&self) -> Vec<&'found Name> {
        self.edges.iter().map(|edge| AsRef::<Name>::as_ref(edge.transition())).collect()
    }

    fn nodes(&self) -> impl Iterator<Item = &'graph Node> + '_ {
        self.start.iter().chain(self.reached.iter().copied())
    }

    fn first(&self) -> Option<&'graph Site> {
        self.edges.first().map(|edge| AsRef::<Site>::as_ref(*edge))
    }
}

impl fmt::Display for Occurrence<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reached: Vec<String> = self.reached.iter().map(ToString::to_string).collect();
        let sites: Vec<String> = self.edges.iter().map(|edge| AsRef::<Site>::as_ref(*edge).to_string()).collect();
        write!(formatter, "{} -> {} at {}", self.start, reached.join(" -> "), sites.join(", "))
    }
}

#[derive(Debug, Default)]
struct Chosen<'graph, 'found> {
    repeats: Vec<Occurrence<'graph, 'found>>,
    used: BTreeSet<&'graph Node>,
}

impl<'graph, 'found> Chosen<'graph, 'found> {
    fn kept(mut self, path: Occurrence<'graph, 'found>) -> Chosen<'graph, 'found> {
        if path.nodes().any(|node| self.used.contains(node)) {
            self
        } else {
            self.used.extend(path.nodes());
            self.repeats.push(path);
            self
        }
    }
}

#[derive(Debug)]
pub(super) struct Motif<'graph, 'found> {
    shape: Vec<&'found Name>,
    repeats: Vec<Occurrence<'graph, 'found>>,
}

impl<'graph, 'found> Motif<'graph, 'found> {
    const SHOWN: usize = 3;

    pub(super) fn repeated(
        shape: Vec<&'found Name>,
        found: Vec<Occurrence<'graph, 'found>>,
    ) -> Option<Motif<'graph, 'found>> {
        let chosen = found.into_iter().fold(Chosen::default(), Chosen::kept);
        (chosen.repeats.len() > 1).then_some(Motif { shape, repeats: chosen.repeats })
    }

    pub(super) fn ranked(mut motifs: Vec<Motif<'graph, 'found>>) -> Vec<Motif<'graph, 'found>> {
        motifs.sort_by(|left, right| {
            right
                .shape
                .len()
                .cmp(&left.shape.len())
                .then_with(|| right.repeats.len().cmp(&left.repeats.len()))
                .then_with(|| left.first().cmp(&right.first()))
        });
        motifs
    }

    fn first(&self) -> Option<&'graph Site> {
        self.repeats.iter().filter_map(Occurrence::first).min()
    }
}

impl From<&Motif<'_, '_>> for Violation {
    fn from(motif: &Motif<'_, '_>) -> Violation {
        let site = motif.first().cloned().unwrap_or_else(|| Site::of_file(std::path::Path::new(".")));
        Violation::new(site, motif)
    }
}

impl fmt::Display for Motif<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shape: Vec<String> = self.shape.iter().map(ToString::to_string).collect();
        let repeats: Vec<String> = self.repeats.iter().take(Self::SHOWN).map(ToString::to_string).collect();
        write!(
            formatter,
            "chain {} repeats over {} disjoint type sets: {}",
            shape.join(" then "),
            self.repeats.len(),
            repeats.join("; ")
        )
    }
}

impl lint_layout::Finding for Motif<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("name the shape once, as a trait or a generic over the types, and implement it per type set")
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    motifs: Count,
    repeats: Count,
}

impl Tally {
    pub(super) fn counted(motifs: &[Motif<'_, '_>]) -> Tally {
        Tally {
            motifs: Count::new(motifs.len()),
            repeats: Count::total(motifs.iter().map(|motif| Count::new(motif.repeats.len()))),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "workspace: {} repeated motifs over {} disjoint type sets", self.motifs, self.repeats)
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("name each repeated shape once, as a trait or a generic")
    }
}

use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::{Fix, Name};
use newtype::Counter;

use crate::count::Count;
use crate::end::{Node, Step};
use crate::graph::Arc;

#[derive(Clone, Debug)]
pub(super) struct Occurrence<'graph, 'found> {
    arcs: Vec<&'graph Arc<'found>>,
    nodes: Vec<&'graph Node>,
}

impl<'graph, 'found> Occurrence<'graph, 'found> {
    pub(super) fn started(arc: &'graph Arc<'found>) -> Occurrence<'graph, 'found> {
        let step: &'graph Step = arc.as_ref();
        Occurrence { arcs: vec![arc], nodes: vec![step.as_ref(), step.reached()] }
    }

    pub(super) fn extended(&self, arc: &'graph Arc<'found>) -> Option<Occurrence<'graph, 'found>> {
        let reached = AsRef::<Step>::as_ref(arc).reached();
        (!self.nodes.contains(&reached)).then(|| Occurrence {
            arcs: self.arcs.iter().copied().chain([arc]).collect(),
            nodes: self.nodes.iter().copied().chain([reached]).collect(),
        })
    }

    pub(super) fn last(&self) -> Option<&'graph Node> {
        self.nodes.last().copied()
    }

    pub(super) fn shape(&self) -> Vec<&'found Name> {
        self.arcs.iter().map(|arc| AsRef::<Name>::as_ref(arc.transition())).collect()
    }

    fn first(&self) -> Option<&'graph Site> {
        self.arcs.first().map(|arc| AsRef::<Site>::as_ref(*arc))
    }
}

impl fmt::Display for Occurrence<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes: Vec<String> = self.nodes.iter().map(ToString::to_string).collect();
        let sites: Vec<String> = self.arcs.iter().map(|arc| AsRef::<Site>::as_ref(*arc).to_string()).collect();
        write!(formatter, "{} at {}", nodes.join(" -> "), sites.join(", "))
    }
}

#[derive(Debug, Default)]
struct Chosen<'graph, 'found> {
    repeats: Vec<Occurrence<'graph, 'found>>,
    used: BTreeSet<&'graph Node>,
}

impl<'graph, 'found> Chosen<'graph, 'found> {
    fn kept(mut self, path: Occurrence<'graph, 'found>) -> Chosen<'graph, 'found> {
        if path.nodes.iter().any(|node| self.used.contains(*node)) {
            self
        } else {
            self.used.extend(path.nodes.iter().copied());
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

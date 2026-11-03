use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::{Fix, Name, Whole, Word, words};
use newtype::Counter;

use crate::count::Count;
use crate::end::{Hop, Node};
use crate::graph::{Consuming, Graph, Hyperedge};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    RoundTrip,
    Tangle,
}

#[derive(Debug)]
pub(super) struct Tangle<'graph, 'found> {
    members: Vec<&'graph Node>,
    sites: Vec<&'graph Site>,
    closing: Vec<&'graph Hyperedge<'found>>,
    kind: Kind,
}

impl<'graph, 'found> Tangle<'graph, 'found> {
    const SHOWN: usize = 8;
    const PAIR: usize = 2;
    const CONVERSIONS: [Word<'static>; 4] = words!["from", "try_from", "from_str", "into"];

    pub(super) fn closed(
        members: BTreeSet<&'graph Node>,
        consuming: &Consuming<'graph, 'found>,
        graph: &'graph Graph<'found>,
    ) -> Tangle<'graph, 'found> {
        let closing: Vec<&'graph Hyperedge<'found>> = members
            .iter()
            .flat_map(|node| consuming.led_within(node, &members))
            .filter(|edge| members.contains(AsRef::<Hop>::as_ref(*edge).reached()))
            .collect();
        let converting = closing
            .iter()
            .find(|edge| AsRef::<Name>::as_ref(edge.transition()).matches::<Whole>(&Self::CONVERSIONS).is_none());
        let kind = match (members.len() == Self::PAIR, converting) {
            (true, None) => Kind::RoundTrip,
            _ => Kind::Tangle,
        };
        let sites = members.iter().filter_map(|node| graph.nodes().get(*node).copied()).collect();
        Tangle { members: members.into_iter().collect(), sites, closing, kind }
    }

    pub(super) fn ranked(mut tangles: Vec<Tangle<'graph, 'found>>) -> Vec<Tangle<'graph, 'found>> {
        tangles.sort_by(|left, right| {
            right.members.len().cmp(&left.members.len()).then_with(|| left.first().cmp(&right.first()))
        });
        tangles
    }

    fn first(&self) -> Option<&'graph Site> {
        self.sites.iter().copied().min()
    }
}

impl From<&Tangle<'_, '_>> for Violation {
    fn from(tangle: &Tangle<'_, '_>) -> Violation {
        let site = tangle
            .first()
            .or_else(|| tangle.closing.first().map(|edge| AsRef::<Site>::as_ref(*edge)))
            .cloned()
            .unwrap_or_else(|| Site::of_file(std::path::Path::new(".")));
        Violation::new(site, tangle)
    }
}

impl fmt::Display for Tangle<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let members: Vec<String> =
            self.members.iter().zip(self.sites.iter()).map(|(node, site)| format!("{node} at {site}")).collect();
        let closing: Vec<String> = self
            .closing
            .iter()
            .take(Self::SHOWN)
            .map(|edge| format!("{} at {}", edge.transition(), AsRef::<Site>::as_ref(*edge)))
            .collect();
        let said = match self.kind {
            Kind::RoundTrip => "round trip",
            Kind::Tangle => "tangle",
        };
        write!(
            formatter,
            "{said} of {} types: {}; joined by {} edges: {}",
            self.members.len(),
            members.join(", "),
            self.closing.len(),
            closing.join(", ")
        )
    }
}

impl lint_layout::Finding for Tangle<'_, '_> {
    fn fix(&self) -> Fix {
        match self.kind {
            Kind::RoundTrip => Fix::new("declare the round trip as a roundtrip! law"),
            Kind::Tangle => Fix::new(
                "break the cycle: declare it as a lifecycle with states!, or split a type so the edges run one way",
            ),
        }
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    tangles: Count,
    round_trips: Count,
    tangled: Count,
}

impl Tally {
    pub(super) fn counted(found: &[Tangle<'_, '_>]) -> Tally {
        let tangles = || found.iter().filter(|tangle| tangle.kind == Kind::Tangle);
        Tally {
            tangles: Count::new(tangles().count()),
            round_trips: Count::new(found.iter().filter(|tangle| tangle.kind == Kind::RoundTrip).count()),
            tangled: Count::total(tangles().map(|tangle| Count::new(tangle.members.len()))),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} tangles holding {} types, {} round trips",
            self.tangles, self.tangled, self.round_trips
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("make the core acyclic: declare lifecycles with states! and round trips with roundtrip! laws")
    }
}

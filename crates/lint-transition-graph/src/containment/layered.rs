use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;

use lint_core::{Fix, Headline, Violation};

use super::cluster::{Cluster, Level, Part};
use super::link::Link;
use super::member::Member;
use super::slice::Slice;
use super::{ClusterLint, Containment, Toward};

#[derive(Debug)]
pub struct AcyclicQuotients;

impl ClusterLint for AcyclicQuotients {
    const HEADLINE: Headline = Headline::new(
        "the parts of each crate and module are layered: collapse the own file and child modules of each crate and module into single nodes of the definition graph (fields, variants and bounds), and no group of them depends on one another in a cycle; signatures and bodies may cycle, as functions do",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.slices()
            .filter(|slice| slice.cluster().level() != Level::Workspace)
            .flat_map(|slice| Cycle::all(view, slice, Toward::Defined))
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
    fn all(view: &Containment<'found, '_>, slice: &Slice<'found>, toward: Toward) -> Vec<Cycle<'found>> {
        let cluster = slice.cluster();
        slice.quotient(view, toward).cycles(|parts, crossing| Cycle::closed(cluster, parts, crossing))
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

#[derive(Debug)]
pub(crate) struct CrateCycles<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    krate: &'view Name,
}

impl<'view, 'found, 'file> CrateCycles<'view, 'found, 'file> {
    pub(crate) fn new(
        view: &'view Containment<'found, 'file>,
        krate: &'view Name,
    ) -> CrateCycles<'view, 'found, 'file> {
        CrateCycles { view, krate }
    }

    fn found(&self, toward: Toward) -> Vec<Cycle<'found>> {
        self.view
            .slices()
            .filter(|slice| slice.cluster().krate().is_some_and(|krate| Borrow::<Name>::borrow(krate) == self.krate))
            .flat_map(|slice| Cycle::all(self.view, slice, toward))
            .filter(|cycle| cycle.members.first().is_some())
            .collect()
    }
}

impl fmt::Display for CrateCycles<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let defined = self.found(Toward::Defined);
        let used = self.found(Toward::Used);
        writeln!(formatter, "  through definitions: {}", defined.len())?;
        defined.iter().try_for_each(|cycle| writeln!(formatter, "    {cycle}"))?;
        writeln!(formatter, "  through uses: {}", used.len())?;
        used.iter().try_for_each(|cycle| writeln!(formatter, "    {cycle}"))
    }
}

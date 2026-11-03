use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;

use crate::count::Count;
use crate::end::Hop;
use crate::graph::Hyperedge;

#[derive(Clone, Copy, Debug)]
pub(super) struct Detour<'graph, 'file> {
    first: &'graph Hyperedge<'graph, 'file>,
    second: &'graph Hyperedge<'graph, 'file>,
}

impl<'graph, 'file> Detour<'graph, 'file> {
    pub(super) fn new(
        first: &'graph Hyperedge<'graph, 'file>,
        second: &'graph Hyperedge<'graph, 'file>,
    ) -> Detour<'graph, 'file> {
        Detour { first, second }
    }
}

impl fmt::Display for Detour<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "through {} by {} at {} and {} at {}",
            AsRef::<Hop>::as_ref(self.first).reached(),
            self.first.transition(),
            AsRef::<Site>::as_ref(self.first),
            self.second.transition(),
            AsRef::<Site>::as_ref(self.second)
        )
    }
}

#[derive(Debug)]
pub(super) struct Triangle<'graph, 'file> {
    direct: &'graph Hyperedge<'graph, 'file>,
    paths: Vec<Detour<'graph, 'file>>,
}

impl<'graph, 'file> Triangle<'graph, 'file> {
    const SHOWN: usize = 3;

    pub(super) fn closing(
        direct: &'graph Hyperedge<'graph, 'file>,
        paths: Vec<Detour<'graph, 'file>>,
    ) -> Option<Triangle<'graph, 'file>> {
        (!paths.is_empty()).then_some(Triangle { direct, paths })
    }
}

impl<'file> From<&Triangle<'_, 'file>> for Violation<'file> {
    fn from(triangle: &Triangle<'_, 'file>) -> Violation<'file> {
        Violation::new(*AsRef::<Site<'file>>::as_ref(triangle.direct), triangle)
    }
}

impl fmt::Display for Triangle<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hop: &Hop = self.direct.as_ref();
        let paths: Vec<String> = self.paths.iter().take(Self::SHOWN).map(ToString::to_string).collect();
        write!(
            formatter,
            "{hop} by {} also goes {} path(s): {}",
            self.direct.transition(),
            self.paths.len(),
            paths.join("; ")
        )
    }
}

impl lint_layout::Finding for Triangle<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("route the direct edge through the path, or declare the law that the two agree")
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    triangles: Count,
    paths: Count,
}

impl Tally {
    pub(super) fn counted(triangles: &[Triangle<'_, '_>]) -> Tally {
        Tally {
            triangles: Count::new(triangles.len()),
            paths: Count::total(triangles.iter().map(|triangle| Count::new(triangle.paths.len()))),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} written edges close a triangle, over {} paths of two edges",
            self.triangles, self.paths
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("route each direct edge through its path, or declare the law that they agree")
    }
}

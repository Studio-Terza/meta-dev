use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;

use crate::count::Count;
use crate::end::{Node, Step};
use crate::graph::Arc;

#[derive(Clone, Copy, Debug)]
pub(super) struct Detour<'graph, 'found> {
    first: &'graph Arc<'found>,
    second: &'graph Arc<'found>,
}

impl<'graph, 'found> Detour<'graph, 'found> {
    pub(super) fn new(first: &'graph Arc<'found>, second: &'graph Arc<'found>) -> Detour<'graph, 'found> {
        Detour { first, second }
    }
}

impl fmt::Display for Detour<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "through {} by {} at {} and {} at {}",
            AsRef::<Step>::as_ref(self.first).reached(),
            self.first.transition(),
            AsRef::<Site>::as_ref(self.first),
            self.second.transition(),
            AsRef::<Site>::as_ref(self.second)
        )
    }
}

#[derive(Debug)]
pub(super) struct Triangle<'graph, 'found> {
    direct: &'graph Arc<'found>,
    paths: Vec<Detour<'graph, 'found>>,
}

impl<'graph, 'found> Triangle<'graph, 'found> {
    const SHOWN: usize = 3;

    pub(super) fn closing(
        direct: &'graph Arc<'found>,
        paths: Vec<Detour<'graph, 'found>>,
    ) -> Option<Triangle<'graph, 'found>> {
        (!paths.is_empty()).then_some(Triangle { direct, paths })
    }
}

impl From<&Triangle<'_, '_>> for Violation {
    fn from(triangle: &Triangle<'_, '_>) -> Violation {
        Violation::new(AsRef::<Site>::as_ref(triangle.direct).clone(), triangle)
    }
}

impl fmt::Display for Triangle<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let step: &Step = self.direct.as_ref();
        let paths: Vec<String> = self.paths.iter().take(Self::SHOWN).map(ToString::to_string).collect();
        write!(
            formatter,
            "{} -> {} by {} also goes {} path(s): {}",
            AsRef::<Node>::as_ref(step),
            step.reached(),
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

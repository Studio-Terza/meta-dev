use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;

use crate::definition::{Declared, Definition};
use crate::link::Link;

#[derive(Debug)]
pub(crate) struct Tangle<'facts, 'file> {
    members: BTreeMap<Definition<'facts>, Site<'file>>,
    closing: Vec<Link<'facts, 'file>>,
}

impl<'facts, 'file> Tangle<'facts, 'file> {
    pub(crate) fn closed(
        members: BTreeSet<Definition<'facts>>,
        links: &[Link<'facts, 'file>],
        declared: &Declared<'facts, 'file>,
    ) -> Tangle<'facts, 'file> {
        let closing = links.iter().filter_map(|link| link.closing(&members)).collect();
        let members =
            members.into_iter().filter_map(|member| declared.site_of(&member).map(|site| (member, site))).collect();
        Tangle { members, closing }
    }

    pub(crate) fn ranked(mut tangles: Vec<Tangle<'facts, 'file>>) -> Vec<Tangle<'facts, 'file>> {
        tangles.sort_by(|left, right| {
            right.members.len().cmp(&left.members.len()).then_with(|| left.first().cmp(&right.first()))
        });
        tangles
    }

    fn first(&self) -> Option<Site<'file>> {
        self.members.values().copied().min()
    }
}

impl<'file> From<&Tangle<'_, 'file>> for Violation<'file> {
    fn from(tangle: &Tangle<'_, 'file>) -> Violation<'file> {
        let site = tangle.first().unwrap_or_else(|| Site::of_file(std::path::Path::new(".")));
        Violation::new(site, tangle)
    }
}

impl fmt::Display for Tangle<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let members: Vec<String> = self.members.iter().map(|(member, site)| format!("{member} at {site}")).collect();
        let closing: Vec<String> = self.closing.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "tangle of {} definitions: {}; closed by {} edges: {}",
            self.members.len(),
            members.join(", "),
            self.closing.len(),
            closing.join(", ")
        )
    }
}

impl lint_layout::Finding for Tangle<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "pull the shared part into its own type, or make the recursion explicit: a type holding itself through Box, Rc or Arc",
        )
    }
}

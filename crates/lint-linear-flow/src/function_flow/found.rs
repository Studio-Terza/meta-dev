use lint_core::{Site, Violation};

use super::finding::Finding;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct Found<'file> {
    site: Site<'file>,
    finding: Finding,
}

impl<'file> Found<'file> {
    pub fn new(site: Site<'file>, finding: Finding) -> Found<'file> {
        Found { site, finding }
    }

    pub fn violation(self, rule: Rule) -> Option<Violation<'file>> {
        Some(self.finding).filter(|finding| finding.rule() == rule).map(|finding| Violation::new(self.site, finding))
    }
}

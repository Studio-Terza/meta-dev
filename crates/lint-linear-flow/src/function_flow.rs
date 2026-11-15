mod found;
mod scan;

use lint_core::{Site, Violation};

use crate::finding::Rule;
use crate::verdict::Verdict;
use found::Found;
pub use scan::Opened;

#[derive(Debug)]
pub struct FunctionFlow<'file> {
    site: Site<'file>,
    shape: Verdict,
    found: Vec<Found<'file>>,
}

impl<'file> FunctionFlow<'file> {
    pub fn verdict(&self) -> Verdict {
        self.found.first().map_or(self.shape, |_| Verdict::Neither)
    }

    pub fn violations(&self, rule: Rule) -> impl Iterator<Item = Violation<'file>> {
        self.found.iter().filter_map(move |found| found.violation(rule))
    }
}

impl<'file> AsRef<Site<'file>> for FunctionFlow<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

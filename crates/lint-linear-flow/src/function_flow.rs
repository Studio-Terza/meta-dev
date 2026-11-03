mod found;
mod root;
mod scan;

use std::path::Path;

use proc_macro2::Span;
use syn::Block;

use lint_core::{Site, Violation};
use lint_layout::LineNumber;

use crate::finding::Rule;
use crate::verdict::Verdict;
use found::Found;
use root::Root;
use scan::Scan;

#[derive(Debug)]
pub struct FunctionFlow<'file> {
    site: Site<'file>,
    shape: Verdict,
    found: Vec<Found<'file>>,
}

impl<'file> FunctionFlow<'file> {
    pub fn judged(path: &'file Path, name: Span, body: &Block) -> FunctionFlow<'file> {
        let root = Root::from(body);
        let mut scan = Scan::new(path);
        root.scanned_by(&mut scan);
        FunctionFlow {
            site: Site::new(path, LineNumber::from(name)),
            shape: root.verdict(),
            found: scan.into_found().into_iter().chain(root.faults(path)).collect(),
        }
    }

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

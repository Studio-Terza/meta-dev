use std::path::Path;

use code_model::{Region, Site};
use lint_core::Violation;
use lint_layout::{LineNumber, Name};

use crate::finding::Finding;
use crate::legacy::{Legacy, Listing};
use crate::lexicon::Identity;

#[derive(Debug)]
pub struct Judged<'model> {
    listing: Listing,
    violation: Violation<'model>,
}

impl<'model> Judged<'model> {
    pub fn new(listing: Listing, violation: Violation<'model>) -> Judged<'model> {
        Judged { listing, violation }
    }

    pub fn listed(self, listing: Listing) -> Option<Violation<'model>> {
        [self.listing].into_iter().find(|judged| *judged == listing).map(|_| self.violation)
    }
}

#[derive(Debug)]
pub struct Candidate {
    identity: Identity,
    line: LineNumber,
    legacy: Legacy,
}

impl Candidate {
    pub fn new(identity: Identity, line: LineNumber, legacy: Legacy) -> Candidate {
        Candidate { identity, line, legacy }
    }

    fn judged<'model>(self, scope: &Scope<'model>) -> Judged<'model> {
        let legacy = Legacy::of_reach(scope.region.reach()).both(scope.eligible).both(self.legacy);
        let finding: Finding = self.identity.finding(scope.context.clone());
        Judged::new(legacy.listing(), Violation::new(Site::new(scope.path, self.line), finding))
    }
}

#[derive(Debug)]
pub struct Scope<'model> {
    path: &'model Path,
    region: Region,
    eligible: Legacy,
    context: &'model Name,
}

impl<'model> Scope<'model> {
    pub fn new(path: &'model Path, region: Region, eligible: Legacy, context: &'model Name) -> Scope<'model> {
        Scope { path, region, eligible, context }
    }

    pub fn judged(&self, candidates: Vec<Candidate>) -> Vec<Judged<'model>> {
        candidates.into_iter().map(|candidate| candidate.judged(self)).collect()
    }
}

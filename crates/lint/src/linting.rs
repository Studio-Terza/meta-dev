use std::mem;
use std::sync::Arc;

use pipeline::{Measure, Pipeline};

use crate::budget::Budget;
use crate::failure::Failure;
use crate::finding::Findings;
use crate::source::Source;
use crate::spent::Spent;
use crate::walk::{Facts, Walker};
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) struct Linting<'run> {
    packages: &'run [Arc<Package>],
}

#[derive(Debug)]
pub(crate) struct Lexed {
    source: Source,
    file: syn::File,
}

#[derive(Debug, Default, Measure)]
pub(crate) struct Judged {
    pub failures: Vec<Failure>,
    pub findings: Findings,
    pub budget: Budget,
    pub spent: Spent,
}

impl<'run> Linting<'run> {
    pub(crate) fn over(packages: &'run [Arc<Package>]) -> Linting<'run> {
        Linting { packages }
    }
}

impl Lexed {
    pub(crate) fn new(source: Source, file: syn::File) -> Lexed {
        Lexed { source, file }
    }
}

impl Judged {
    pub(crate) fn failed(failure: Failure) -> Judged {
        Judged { failures: vec![failure], ..Judged::default() }
    }
}

impl Pipeline for Linting<'_> {
    type Unit = Lexed;
    type Parsed = Facts;
    type Measured = Judged;
    type Act = Judged;

    fn parsed(&self, unit: Lexed) -> Facts {
        Walker::walked(&unit.source, &unit.file)
    }

    fn measured(&self, mut whole: Facts) -> Judged {
        let budget = mem::take(&mut whole.test_budget);
        Judged { findings: Findings::from(whole.findings(self.packages)), budget, ..Judged::default() }
    }

    fn act(&self, measured: Judged) -> Judged {
        measured
    }
}

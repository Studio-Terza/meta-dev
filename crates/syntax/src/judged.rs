use std::io::{self, Write};
use std::process::ExitCode;

use pipeline::Measure;

use crate::budget::Budget;
use crate::failure::Failure;
use crate::finding::Findings;
use crate::reading::{Read, Reader};
use crate::spent::{Clock, Spent, Stage};
use crate::tree::Tree;
use crate::walk::Walker;
use crate::workspace::Workspace;

pipeline::measured! {
    #[derive(Debug)]
    pub struct Judged {
        failures: Vec<Failure>,
        findings: Findings,
        budget: Budget,
        spent: Spent,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Violations,
    Refused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Gate,
    Timings,
}

impl Judged {
    #[must_use]
    pub fn failed(failure: Failure) -> Judged {
        Judged { failures: vec![failure], ..Judged::default() }
    }

    pub fn of(clock: &(impl Clock + Sync), find: impl FnOnce() -> Result<Tree, Failure>) -> Judged {
        let mut spent = Spent::default();
        let found = spent.timed(clock, Stage::Find, || {
            find().and_then(|tree| Workspace::found(&tree).map(|workspace| (tree, workspace)))
        });
        match found {
            Ok((tree, workspace)) => {
                let packages = workspace.packages().to_vec();
                let Read { failures, facts, spent: reading } = Reader::new(clock, &tree).read(workspace.into_roots());
                let budget = facts.test_budget;
                let findings = spent.timed(clock, Stage::Reduce, || Findings::from(facts.findings(&packages)));
                Judged { failures, findings, budget, spent: spent.joined(reading) }
            }
            Err(failure) => Judged { spent, ..Judged::failed(failure) },
        }
    }

    #[must_use]
    pub fn timed(self, clock: &impl Clock, run: impl FnOnce() -> Judged) -> Judged {
        let mut judged = self;
        let whole = judged.spent.timed(clock, Stage::Run, run);
        judged.joined(whole)
    }

    #[must_use]
    pub fn decided(&self) -> Verdict {
        let failed = self.failures.first().map(|_| Verdict::Refused);
        let violated = self.findings.first().map(|_| ()).or(self.budget.exceeded().map(|_| ()));
        failed.or(violated.map(|()| Verdict::Violations)).unwrap_or(Verdict::Clean)
    }

    pub fn written(self, shown: Shown, out: &mut impl Write, err: &mut impl Write) -> io::Result<()> {
        let Judged { failures, findings, budget, spent } = self;
        let mut said: Vec<String> = failures.iter().map(ToString::to_string).collect();
        said.sort();
        said.iter().try_for_each(|failure| writeln!(err, "{failure}"))?;
        findings.written(&Walker::headlines(), err)?;
        budget.exceeded().map_or_else(|| writeln!(out, "{budget}"), |exceeded| writeln!(err, "{exceeded}"))?;
        match shown {
            Shown::Timings => spent.written(out),
            Shown::Gate => Ok(()),
        }
    }
}

impl From<Verdict> for ExitCode {
    fn from(verdict: Verdict) -> ExitCode {
        match verdict {
            Verdict::Clean => ExitCode::SUCCESS,
            Verdict::Violations | Verdict::Refused => ExitCode::FAILURE,
        }
    }
}

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::Files;
use invariants::Ignoring;
use pipeline::Measure;
use port_core::Run;
use rayon::ThreadPoolBuilder;
use timer_ports::{Timer, TimerFiller};

use crate::budget::Budget;
use crate::failure::Failure;
use crate::finding::Findings;
use crate::ignored::Ignored;
use crate::reading::{Read, Reader};
use crate::spent::{Spent, Stage};
use crate::tree::Tree;
use crate::walk::Walker;
use crate::workspace::Workspace;

pipeline::measured! {
    #[derive(Debug)]
    struct Judged {
        failures: Vec<Failure>,
        findings: Findings,
        budget: Budget,
        spent: Spent,
    }
}

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
    mode: Mode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Gate,
    Timings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Clean,
    Violations,
    Refused,
}

impl Cli {
    const TIMINGS: &'static str = "--timings";
    const CURRENT: &'static str = ".";
    const STACK_BYTES: usize = 256 * 1024 * 1024;

    pub fn from_args(arguments: impl IntoIterator<Item = OsString>) -> Cli {
        let given: Vec<OsString> = arguments.into_iter().collect();
        let mode = given.iter().find(|argument| *argument == Cli::TIMINGS).map_or(Mode::Gate, |_| Mode::Timings);
        let named: Vec<PathBuf> =
            given.iter().filter(|argument| *argument != Cli::TIMINGS).map(PathBuf::from).collect();
        let roots = match named.first() {
            Some(_) => named,
            None => vec![PathBuf::from(Cli::CURRENT)],
        };
        Cli { roots, mode }
    }

    pub fn verdict(
        &self,
        files: &(impl Files + Sync),
        working: &Path,
        timer: impl TimerFiller + Sync,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        let run = Run::root();
        let clock = run.port(timer);
        let judged = match ThreadPoolBuilder::new().stack_size(Cli::STACK_BYTES).build() {
            Ok(pool) => pool.install(|| self.read(files, working, &clock)),
            Err(failure) => Judged::failed(Failure::Threads(failure)),
        };
        let decided = Cli::decided(&judged);
        match self.written(judged, out, err) {
            Ok(()) => decided,
            Err(failure) => {
                writeln!(err, "cannot write the output: {failure}").ignoring(Ignored::Unwritable);
                Verdict::Refused
            }
        }
    }

    fn read(&self, files: &(impl Files + Sync), working: &Path, timer: &(impl Timer + Sync)) -> Judged {
        let mut judged = Judged::default();
        let whole = judged.spent.timed(timer, Stage::Run, || {
            Judged::total(self.roots.iter().map(|root| Cli::tree(files, working, timer, root)))
        });
        judged.joined(whole)
    }

    fn tree(files: &(impl Files + Sync), working: &Path, timer: &(impl Timer + Sync), root: &Path) -> Judged {
        let mut spent = Spent::default();
        let found = spent.timed(timer, Stage::Find, || {
            Tree::read(files, working, root).and_then(|tree| Workspace::found(&tree).map(|workspace| (tree, workspace)))
        });
        match found {
            Ok((tree, workspace)) => {
                let packages = workspace.packages().to_vec();
                let Read { failures, facts, spent: reading } = Reader::new(timer, &tree).read(workspace.into_roots());
                let budget = facts.test_budget;
                let findings = spent.timed(timer, Stage::Reduce, || Findings::from(facts.findings(&packages)));
                Judged { failures, findings, budget, spent: spent.joined(reading) }
            }
            Err(failure) => Judged { spent, ..Judged::failed(failure) },
        }
    }

    fn decided(judged: &Judged) -> Verdict {
        let failed = judged.failures.first().map(|_| Verdict::Refused);
        let violated = judged.findings.first().map(|_| ()).or(judged.budget.exceeded().map(|_| ()));
        failed.or(violated.map(|()| Verdict::Violations)).unwrap_or(Verdict::Clean)
    }

    fn written(&self, judged: Judged, out: &mut impl Write, err: &mut impl Write) -> io::Result<()> {
        let Judged { failures, findings, budget, spent } = judged;
        let mut said: Vec<String> = failures.iter().map(ToString::to_string).collect();
        said.sort();
        said.iter().try_for_each(|failure| writeln!(err, "{failure}"))?;
        findings.written(&Walker::headlines(), err)?;
        budget.exceeded().map_or_else(|| writeln!(out, "{budget}"), |exceeded| writeln!(err, "{exceeded}"))?;
        match self.mode {
            Mode::Timings => spent.written(out),
            Mode::Gate => Ok(()),
        }
    }
}

impl Judged {
    fn failed(failure: Failure) -> Judged {
        Judged { failures: vec![failure], ..Judged::default() }
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

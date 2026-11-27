use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::Files;
use invariants::Ignoring;
use port_core::Run;
use rayon::ThreadPoolBuilder;
use timer_ports::{Timer, TimerFiller};

use crate::failure::Failure;
use crate::finding::Findings;
use crate::ignored::Ignored;
use crate::reading::{Read, Reader};
use crate::spent::Stage;
use crate::workspace::{Tree, Workspace};

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

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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
        timer: impl TimerFiller + Sync,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> Verdict {
        let run = Run::root();
        let clock = run.port(timer);
        let read = match ThreadPoolBuilder::new().stack_size(Cli::STACK_BYTES).build() {
            Ok(pool) => pool.install(|| self.read(files, &clock)),
            Err(failure) => Read::failed(Failure::Threads(failure)),
        };
        let decided = Cli::decided(&read);
        match self.written(read, out, err) {
            Ok(()) => decided,
            Err(failure) => {
                writeln!(err, "cannot write the output: {failure}").ignoring(Ignored::Unwritable);
                Verdict::Refused
            }
        }
    }

    fn read(&self, files: &(impl Files + Sync), timer: &(impl Timer + Sync)) -> Read {
        let mut read = Read::default();
        let whole = read.spent.timed(timer, Stage::Run, || {
            self.roots.iter().map(|root| Cli::tree(files, timer, root)).fold(Read::default(), Read::joined)
        });
        read.joined(whole)
    }

    fn tree(files: &(impl Files + Sync), timer: &(impl Timer + Sync), root: &Path) -> Read {
        let mut read = Read::default();
        let found = read.spent.timed(timer, Stage::Find, || {
            Tree::opened(files, root).and_then(|tree| Workspace::found(files, &tree).map(|workspace| (tree, workspace)))
        });
        match found {
            Ok((tree, workspace)) => {
                let packages = workspace.packages().to_vec();
                let sources = Reader::new(files, timer, &tree).read(workspace.into_roots());
                let mut reduced = read.joined(sources);
                let facts = std::mem::take(&mut reduced.facts);
                let findings = reduced.spent.timed(timer, Stage::Reduce, || facts.reduced(&packages));
                reduced.findings.extend(findings);
                reduced
            }
            Err(failure) => read.joined(Read::failed(failure)),
        }
    }

    fn decided(read: &Read) -> Verdict {
        let failed = read.failures.first().map(|_| Verdict::Refused);
        let violated = read.findings.first().map(|_| ()).or(read.budget.exceeded().map(|_| ()));
        failed.or(violated.map(|()| Verdict::Violations)).unwrap_or(Verdict::Clean)
    }

    fn written(&self, read: Read, out: &mut impl Write, err: &mut impl Write) -> io::Result<()> {
        let Read { failures, findings, budget, spent, .. } = read;
        let mut said: Vec<String> = failures.iter().map(ToString::to_string).collect();
        said.sort();
        said.iter().try_for_each(|failure| writeln!(err, "{failure}"))?;
        findings.into_iter().collect::<Findings>().written(err)?;
        budget.exceeded().map_or_else(|| writeln!(out, "{budget}"), |exceeded| writeln!(err, "{exceeded}"))?;
        match self.mode {
            Mode::Timings => spent.written(out),
            Mode::Gate => Ok(()),
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

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::sync::OnceLock;

use algebra::Semigroup;
use lint_reader::Partition;
use timer_ports::{Elapsed, Moment, Timer};

mod boundary;

pub(crate) use boundary::Named;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Timings {
    Shown,
    Hidden,
}

impl Timings {
    const FLAG: &'static str = "--timings";

    pub(crate) fn of_arguments(arguments: &[OsString]) -> Timings {
        arguments.iter().find(|argument| argument.as_os_str() == Self::FLAG).map_or(Timings::Hidden, |_| Timings::Shown)
    }

    pub(crate) fn unflagged(arguments: Vec<OsString>) -> Vec<OsString> {
        arguments.into_iter().filter(|argument| argument.as_os_str() != Self::FLAG).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Worker(usize);

impl Worker {
    pub(crate) const MAIN: Worker = Worker(0);
}

impl From<usize> for Worker {
    fn from(index: usize) -> Worker {
        Worker(index)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Step {
    Total,
    Stage,
    Locate,
    Read(Worker),
    Lower(Worker),
    Judged(Worker),
    Model,
    Contexts,
    Lint(Named),
    Waited(Waited),
}

impl Step {
    pub(crate) fn waited(self) -> Step {
        match self {
            Step::Model => Step::Waited(Waited::Model),
            Step::Judged(worker) => Step::Waited(Waited::Judged(worker)),
            Step::Lint(name) => Step::Waited(Waited::Lint(name)),
            Step::Total
            | Step::Stage
            | Step::Locate
            | Step::Read(_)
            | Step::Lower(_)
            | Step::Contexts
            | Step::Waited(_) => self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Waited {
    Model,
    Judged(Worker),
    Lint(Named),
}

impl fmt::Display for Waited {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Waited::Model => formatter.write_str("wait model: until cargo metadata has described the workspaces"),
            Waited::Judged(Worker(index)) => {
                write!(formatter, "wait judged lints on worker {index}: until the context-lint parts are joined")
            }
            Waited::Lint(name) => {
                write!(
                    formatter,
                    "wait lint {name}: until the model, the contexts and the workspaces it reads are ready"
                )
            }
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Step::Total => formatter.write_str("phase total"),
            Step::Stage => formatter.write_str("phase stage: list the trees and their packages"),
            Step::Locate => formatter.write_str("phase locate: describe the workspaces through cargo metadata"),
            Step::Read(Worker(index)) => write!(formatter, "phase read: read and parse the sources on worker {index}"),
            Step::Lower(Worker(index)) => {
                write!(formatter, "phase lower: lower and file-lint its share of files on worker {index}")
            }
            Step::Judged(Worker(index)) => {
                write!(formatter, "phase judged lints: judge its share of files on worker {index}")
            }
            Step::Model => formatter.write_str("phase model: merge the lowered files into the code model"),
            Step::Contexts => formatter.write_str("phase contexts: join every file's context-lint parts"),
            Step::Lint(name) => write!(formatter, "lint {name}"),
            Step::Waited(waited) => write!(formatter, "{waited}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Timing {
    step: Step,
    began: Moment,
    took: Elapsed,
}

impl Semigroup for Timing {
    fn combine(self, later: Timing) -> Timing {
        Timing { step: self.step, began: self.began.min(later.began), took: self.took + later.took }
    }
}

#[derive(Debug)]
pub(crate) struct Measured<Done> {
    done: Done,
    timing: Timing,
}

impl<Done> Measured<Done> {
    pub(crate) fn kept(self, cell: &OnceLock<Done>) -> Timing {
        let timing = self.timing;
        cell.get_or_init(|| self.done);
        timing
    }

    pub(crate) fn done(&self) -> &Done {
        &self.done
    }

    pub(crate) fn timing(&self) -> Timing {
        self.timing
    }
}

impl<Done, Failure> Measured<Result<Done, Failure>> {
    pub(crate) fn transposed(self) -> Result<Measured<Done>, Failure> {
        let timing = self.timing;
        self.done.map(|done| Measured { done, timing })
    }
}

#[derive(Debug)]
pub(crate) struct Stopwatch<'timer, T> {
    timer: &'timer T,
    started: Moment,
}

impl<'timer, T: Timer> Stopwatch<'timer, T> {
    pub(crate) fn started(timer: &'timer T) -> Stopwatch<'timer, T> {
        Stopwatch { timer, started: timer.now() }
    }

    pub(crate) fn stopped(&self, step: Step) -> Timing {
        Timing { step, began: self.started, took: self.timer.now().since(self.started) }
    }

    pub(crate) fn measured<Done>(timer: &'timer T, step: Step, work: impl FnOnce() -> Done) -> Measured<Done> {
        let stopwatch = Stopwatch::started(timer);
        let done = work();
        Measured { done, timing: stopwatch.stopped(step) }
    }
}

#[derive(Debug)]
pub(crate) struct Table {
    shown: Timings,
    timings: Vec<Timing>,
}

impl Table {
    pub(crate) fn new(shown: Timings, timings: Vec<Timing>) -> Table {
        let summed = timings.into_iter().fold(BTreeMap::new(), |mut steps: BTreeMap<String, Timing>, timing| {
            let held = steps.remove(&timing.step.to_string()).map_or(timing, |earlier| earlier.combine(timing));
            steps.insert(held.step.to_string(), held);
            steps
        });
        Table { shown, timings: summed.into_values().collect() }
    }

    fn rows(&self) -> Vec<&Timing> {
        let mut rows: Vec<&Timing> = self.timings.iter().collect();
        rows.sort_by_key(|timing| Reverse(timing.took));
        rows
    }
}

impl fmt::Display for Table {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.shown {
            Timings::Shown => Rows(&self.rows()).fmt(formatter),
            Timings::Hidden => Ok(()),
        }
    }
}

struct Rows<'table>(&'table [&'table Timing]);

impl fmt::Display for Rows<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "lint timings in milliseconds, took then started:")?;
        self.0
            .iter()
            .try_for_each(|timing| writeln!(formatter, "{:>7} {:>7}  {}", timing.took, timing.began, timing.step))
    }
}

impl From<Worker> for Partition {
    fn from(worker: Worker) -> Partition {
        Partition::from(worker.0)
    }
}

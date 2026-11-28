use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use file_ports::Files;
use invariants::Ignoring;
use pipeline::Measure;
use port_core::Run;
use syntax::{Judged, Pool, Shown};
use timer_ports::TimerFiller;

use crate::disk::{Disked, Timed};
use crate::ignored::Ignored;

pub use syntax::Verdict;

#[derive(Debug)]
pub struct Cli {
    roots: Vec<PathBuf>,
    shown: Shown,
}

impl Cli {
    const TIMINGS: &'static str = "--timings";
    const CURRENT: &'static str = ".";

    pub fn from_args(arguments: impl IntoIterator<Item = OsString>) -> Cli {
        let given: Vec<OsString> = arguments.into_iter().collect();
        let shown = given.iter().find(|argument| *argument == Cli::TIMINGS).map_or(Shown::Gate, |_| Shown::Timings);
        let named: Vec<PathBuf> =
            given.iter().filter(|argument| *argument != Cli::TIMINGS).map(PathBuf::from).collect();
        let roots = match named.first() {
            Some(_) => named,
            None => vec![PathBuf::from(Cli::CURRENT)],
        };
        Cli { roots, shown }
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
        let port = run.port(timer);
        let clock = Timed::new(&port);
        let disked = Disked::new(files);
        let read = || {
            Judged::default().timed(&clock, || {
                Judged::total(self.roots.iter().map(|root| Judged::of(&clock, || disked.tree(working, root))))
            })
        };
        let judged = match Pool::installed(read) {
            Ok(judged) => judged,
            Err(failure) => Judged::failed(failure),
        };
        let decided = judged.decided();
        match judged.written(self.shown, out, err) {
            Ok(()) => decided,
            Err(failure) => {
                writeln!(err, "cannot write the output: {failure}").ignoring(Ignored::Unwritable);
                Verdict::Refused
            }
        }
    }
}

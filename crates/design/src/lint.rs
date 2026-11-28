mod ignored;

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use file_ports::{Disked, Files};
use invariants::Ignoring;
use pipeline::Measure;
use port_core::Run;
use syntax::{Judged, Pool, Shown};
use timer_ports::{Timed, TimerFiller};

use ignored::Ignored;
pub use syntax::Verdict;

#[derive(Debug)]
pub struct Lint {
    roots: Vec<PathBuf>,
    shown: Shown,
}

impl Lint {
    const LINT: &'static str = "lint";
    const TIMINGS: &'static str = "--timings";
    const CURRENT: &'static str = ".";

    #[must_use]
    pub fn asked(arguments: &[OsString]) -> Option<Lint> {
        match arguments {
            [first, rest @ ..] if first == Lint::LINT => Some(Lint::from_args(rest.iter().cloned())),
            [..] => None,
        }
    }

    pub fn from_args(arguments: impl IntoIterator<Item = OsString>) -> Lint {
        let given: Vec<OsString> = arguments.into_iter().collect();
        let shown = given.iter().find(|argument| *argument == Lint::TIMINGS).map_or(Shown::Gate, |_| Shown::Timings);
        let named: Vec<PathBuf> =
            given.iter().filter(|argument| *argument != Lint::TIMINGS).map(PathBuf::from).collect();
        let roots = match named.first() {
            Some(_) => named,
            None => vec![PathBuf::from(Lint::CURRENT)],
        };
        Lint { roots, shown }
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

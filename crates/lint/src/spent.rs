use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};
use std::mem;

use pipeline::Measure;
use timer_ports::{Elapsed, Timer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Stage {
    Run,
    Find,
    Parse,
    Visit,
    Reduce,
}

#[derive(Debug, Default, Measure)]
pub(crate) struct Spent(BTreeMap<Stage, Elapsed>);

impl Spent {
    pub(crate) fn timed<Made>(&mut self, timer: &impl Timer, stage: Stage, work: impl FnOnce() -> Made) -> Made {
        let start = timer.now();
        let made = work();
        let spent = Spent(BTreeMap::from([(stage, timer.now().since(start))]));
        *self = mem::take(self).joined(spent);
        made
    }

    pub(crate) fn written(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "lint timings in milliseconds, each stage summed over files and threads:")?;
        self.0.iter().try_for_each(|(stage, elapsed)| writeln!(out, "{elapsed:>7}  {stage}"))
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Run => formatter.write_str("run: wall time of the whole run"),
            Stage::Find => formatter.write_str("find: list the packages and their crate roots"),
            Stage::Parse => formatter.write_str("parse: read and parse each file once"),
            Stage::Visit => {
                formatter.write_str("visit: walk each file's tree once through every lint and the test budget")
            }
            Stage::Reduce => formatter.write_str("reduce: join the graph lints' facts"),
        }
    }
}

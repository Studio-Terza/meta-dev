use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};
use std::mem;
use std::time::Duration;

use pipeline::Measure;

pub trait Clock {
    fn now(&self) -> Duration;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Run,
    Find,
    Parse,
    Visit,
    Reduce,
}

pipeline::measured! {
    #[derive(Debug)]
    pub struct Spent(BTreeMap<Stage, Duration>);
}

impl Spent {
    pub fn timed<Made>(&mut self, clock: &impl Clock, stage: Stage, work: impl FnOnce() -> Made) -> Made {
        let start = clock.now();
        let made = work();
        let spent = Spent(BTreeMap::from([(stage, clock.now().saturating_sub(start))]));
        *self = mem::take(self).joined(spent);
        made
    }

    pub fn written(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "lint timings in microseconds, each stage summed over files and threads:")?;
        self.0.iter().try_for_each(|(stage, spent)| writeln!(out, "{:>10}  {stage}", spent.as_micros()))
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Run => formatter.write_str("run: wall time of the whole run"),
            Stage::Find => formatter.write_str("find: read the tree once and list its packages and crate roots"),
            Stage::Parse => formatter.write_str("parse: parse each file once"),
            Stage::Visit => {
                formatter.write_str("visit: walk each file's tree once through every lint and the test budget")
            }
            Stage::Reduce => formatter.write_str("reduce: join the graph lints' facts"),
        }
    }
}

use crate::cli::request::command::printed::Lines;
mod boundary;
mod placed;
mod shown;

use design_model::{JobId, NodeId, Prefix};
use design_runs::RunsOf;
use store_ports::Store;
use structurer::SegmentRef;

use crate::cli::failure::Failure;
use crate::cli::output::{Attempted, Line};
use console_ports::Shown as Printed;
pub use placed::Found;
use shown::Shown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Node(NodeId),
    Message(Prefix),
    Segment(SegmentRef),
    Job(JobId),
}

impl Subject {
    pub fn shown(self, store: &impl Store) -> Result<Printed, Failure> {
        match self {
            Subject::Node(id) => Found::find(store, id).and_then(|found| found.printed(store)),
            Subject::Message(prefix) => Shown::message(store, &prefix).and_then(|shown| shown.printed(store)),
            Subject::Segment(reference) => Shown::segment(store, &reference).and_then(|shown| shown.printed(store)),
            Subject::Job(job) => Subject::ran(store, job),
        }
    }

    fn ran(store: &impl Store, job: JobId) -> Result<Printed, Failure> {
        let ran = store.read(RunsOf::from(job))?;
        ran.first().ok_or(Failure::Unrun(job))?;
        let lines = ran.iter().map(|one| {
            Line::Ran(Attempted::new(one.identity(), job, one.state(), one.usage()).replied(one.reply().cloned()))
        });
        Ok(Printed::said(Lines::from(&lines.collect::<Vec<Line>>())))
    }
}

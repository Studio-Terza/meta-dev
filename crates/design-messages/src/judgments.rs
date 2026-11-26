mod boundary;

use port_core::Op;

pub use boundary::{Evidence, Judgment, LineNumber, Revision, Ruling, SourcePath};

#[derive(Debug, Op)]
pub struct Judged(Vec<Judgment>);

#[derive(Clone, Copy, Debug, Op)]
pub struct Judgments;

impl From<Vec<Judgment>> for Judged {
    fn from(judgments: Vec<Judgment>) -> Judged {
        Judged(judgments)
    }
}

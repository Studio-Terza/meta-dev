use grammar::test_summary::Summary;

use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finished(Seconds);

impl Finished {
    pub fn seconds(self) -> Seconds {
        self.0
    }
}

impl From<Summary> for Finished {
    fn from(summary: Summary) -> Finished {
        Finished(Seconds::new(summary.seconds()))
    }
}

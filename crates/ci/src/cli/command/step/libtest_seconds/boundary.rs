use std::path::Path;

use grammar_test_summary::Summary;
use text_lines::LossyText;

use crate::cli::command::failure::Failure;
use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finished(Seconds);

impl Finished {
    pub fn seconds(self) -> Seconds {
        self.0
    }

    pub fn read<'path>(path: &'path Path, line: &str) -> Option<Result<Finished, Failure<'path>>> {
        Summary::opened(line).map(|opened| {
            opened
                .summary()
                .map(|summary| Finished(Seconds::new(summary.seconds())))
                .ok_or_else(|| Failure::MalformedTestResult { path, line: LossyText::from(line.as_bytes()) })
        })
    }
}

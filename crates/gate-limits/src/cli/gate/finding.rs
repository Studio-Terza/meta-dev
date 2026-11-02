use std::fmt;

use crate::cli::failure::Failure;
use crate::cli::gate::limit::Limit;
use crate::cli::gate::record::Entry;
use crate::cli::gate::seconds::Seconds;
use crate::cli::gate::verdict::Verdict;
use crate::cli::stage::Stage;

#[derive(Debug)]
pub struct Finding {
    stage: Stage,
    measured: Seconds,
    limit: Limit,
    verdict: Verdict,
}

impl Finding {
    #[must_use]
    pub fn new(stage: Stage, measured: Seconds, limit: Limit) -> Finding {
        Finding { stage, measured, limit, verdict: limit.judge(measured) }
    }

    #[must_use]
    pub fn blocks(&self) -> bool {
        self.verdict == Verdict::Block
    }

    #[must_use]
    pub fn past_warn(self) -> Option<Finding> {
        (self.verdict != Verdict::Within).then_some(self)
    }

    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    pub fn lowered(self) -> Result<Entry, Failure> {
        self.limit.lowered(&self.stage, self.measured).map(|seconds| Entry::new(self.stage, seconds))
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {} took {}, {}", self.verdict, self.stage, self.measured, self.limit)
    }
}

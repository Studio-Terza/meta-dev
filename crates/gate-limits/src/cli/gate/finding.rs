use std::fmt;

use crate::cli::gate::limit::Limit;
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
    pub fn new(stage: Stage, measured: Seconds, limit: Limit) -> Finding {
        Finding { stage, measured, limit, verdict: limit.judge(measured) }
    }

    pub fn blocks(&self) -> bool {
        self.verdict == Verdict::Block
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {} took {}, {}", self.verdict, self.stage, self.measured, self.limit)
    }
}

use std::fmt;

use super::entry::Entry;
use crate::failure::Failure;
use crate::gate::limit::Limit;
use crate::gate::verdict::Verdict;
use crate::seconds::Seconds;
use crate::stage::Stage;

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
    pub fn blocks(&self) -> Option<&Finding> {
        (self.verdict == Verdict::Block).then_some(self)
    }

    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    pub fn lowered(self) -> Result<Entry, Failure<'static>> {
        match self.limit.lowered(self.measured) {
            Ok(seconds) => Ok(Entry::new(self.stage, seconds)),
            Err(refusal) => Err(refusal.refusing(self.stage, self.measured)),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {} took {}, {}", self.verdict, self.stage, self.measured, self.limit)
    }
}

impl AsRef<Verdict> for Finding {
    fn as_ref(&self) -> &Verdict {
        &self.verdict
    }
}

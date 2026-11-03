use std::fmt;

use crate::cli::failure::Failure;
use crate::cli::gate::limit::Limit;
use crate::cli::gate::record::Entry;
use crate::cli::gate::seconds::Seconds;
use crate::cli::gate::verdict::Verdict;
use crate::cli::stage::Stage;

#[derive(Debug)]
pub struct Blocking<'finding>(&'finding Finding);

#[derive(Debug)]
pub struct Finding {
    stage: Stage,
    measured: Seconds,
    limit: Limit,
    verdict: Verdict,
}

impl Blocking<'_> {
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.0.stage
    }
}

impl Finding {
    pub(crate) fn new(stage: &Stage, limit: Option<&Limit>, measured: Option<&Seconds>) -> Result<Finding, Failure> {
        let limit =
            limit.copied().and_then(Limit::ordered).ok_or_else(|| Failure::Unlimited { stage: stage.clone() })?;
        let measured = *measured.ok_or_else(|| Failure::Unmeasured { stage: stage.clone() })?;
        Ok(Finding { stage: stage.clone(), measured, limit, verdict: limit.judge(measured) })
    }

    #[must_use]
    pub fn blocks(&self) -> Option<Blocking<'_>> {
        (self.verdict == Verdict::Block).then_some(Blocking(self))
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

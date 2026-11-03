use std::collections::BTreeMap;

use crate::cli::failure::Failure;
use crate::cli::gate::finding::Finding;
use crate::cli::gate::limit::Limit;
use crate::cli::gate::seconds::Seconds;
use crate::cli::stage::Stage;

#[derive(Debug, Default)]
pub struct Pairing {
    limit: Option<Limit>,
    measured: Option<Seconds>,
}

impl Pairing {
    pub fn joined(limits: BTreeMap<Stage, Limit>, measured: BTreeMap<Stage, Seconds>) -> BTreeMap<Stage, Pairing> {
        measured.into_iter().fold(
            limits.into_iter().map(|(stage, limit)| (stage, Pairing { limit: Some(limit), measured: None })).collect(),
            |mut joined, (stage, seconds)| {
                joined.entry(stage).or_default().measured = Some(seconds);
                joined
            },
        )
    }

    pub fn finding(self, stage: Stage) -> Result<Finding, Failure<'static>> {
        match (self.limit.and_then(Limit::ordered), self.measured) {
            (None, _) => Err(Failure::Unlimited { stage }),
            (Some(_), None) => Err(Failure::Unmeasured { stage }),
            (Some(limit), Some(measured)) => Ok(Finding::new(stage, measured, limit)),
        }
    }
}

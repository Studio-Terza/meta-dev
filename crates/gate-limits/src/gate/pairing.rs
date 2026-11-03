use std::collections::BTreeMap;

use crate::failure::Failure;
use crate::gate::finding::Finding;
use crate::gate::limit::Limit;
use crate::seconds::Seconds;
use crate::stage::Stage;

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
        match self.limit.and_then(Limit::ordered) {
            None => Err(Failure::Unlimited { stage }),
            Some(limit) => self.measured_against(stage, limit),
        }
    }

    fn measured_against(self, stage: Stage, limit: Limit) -> Result<Finding, Failure<'static>> {
        match self.measured {
            None => Err(Failure::Unmeasured { stage }),
            Some(measured) => Ok(Finding::new(stage, measured, limit)),
        }
    }
}

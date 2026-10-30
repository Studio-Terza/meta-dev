use crate::ids::{CheckName, RunCount, Sha};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pull {
    pub head: Sha,
    pub required: Vec<CheckName>,
    pub reported: Vec<Check>,
    pub runs: RunCount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: CheckName,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Green,
    Red,
    Pending,
}

impl Pull {
    pub fn outcome(&self, name: &CheckName) -> Option<Outcome> {
        self.reported.iter().rev().find(|check| check.name == *name).map(|check| check.outcome)
    }
}

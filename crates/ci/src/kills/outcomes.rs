use std::path::{Path, PathBuf};

use json_ports::Json;

use crate::failure::Failure;
use crate::kills::log::Log;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct Mutant(String);

#[derive(Debug)]
pub(crate) struct Shard;

#[derive(Debug)]
pub(crate) struct Ran {
    scenario: Scenario,
    summary: Summary,
    log: Log,
}

#[derive(Debug)]
pub(crate) enum Fate<'outcome> {
    Inventory,
    Caught(&'outcome Mutant),
    Missed(&'outcome Mutant),
    Unviable(&'outcome Mutant),
}

#[derive(Debug, serde::Deserialize)]
struct Outcomes {
    outcomes: Vec<Outcome>,
}

#[derive(Debug, serde::Deserialize)]
struct Outcome {
    scenario: Scenario,
    summary: Summary,
    log_path: PathBuf,
}

#[derive(Debug, serde::Deserialize)]
enum Scenario {
    Baseline,
    Mutant(Named),
}

#[derive(Debug, serde::Deserialize)]
struct Named {
    name: Mutant,
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
enum Summary {
    Success,
    CaughtMutant,
    MissedMutant,
    Unviable,
    Failure,
    Timeout,
}

impl Shard {
    const OUTCOMES: &'static str = "outcomes.json";

    pub(crate) fn read(directory: &Path, text: impl Fn(&Path) -> Result<String, Failure>) -> Result<Vec<Ran>, Failure> {
        let path = directory.join(Shard::OUTCOMES);
        let read = text(&path)?;
        let outcomes: Outcomes = Json::read(&read).map_err(|source| Failure::Outcomes { path, source })?;
        outcomes.outcomes.into_iter().map(|outcome| outcome.ran(directory, &text)).collect()
    }
}

impl Outcome {
    fn ran(self, directory: &Path, text: impl Fn(&Path) -> Result<String, Failure>) -> Result<Ran, Failure> {
        let log = Log::read(&text(&directory.join(&self.log_path))?);
        Ok(Ran { scenario: self.scenario, summary: self.summary, log })
    }
}

impl Ran {
    pub(crate) fn log(&self) -> &Log {
        &self.log
    }

    pub(crate) fn fate(&self) -> Fate<'_> {
        match &self.scenario {
            Scenario::Baseline => Fate::Inventory,
            Scenario::Mutant(Named { name }) => match self.summary {
                Summary::CaughtMutant | Summary::Timeout => Fate::Caught(name),
                Summary::MissedMutant => Fate::Missed(name),
                Summary::Unviable => Fate::Unviable(name),
                Summary::Success | Summary::Failure => Fate::Inventory,
            },
        }
    }
}

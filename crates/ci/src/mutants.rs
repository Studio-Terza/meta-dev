use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use grammar_lines::{Fields, Lines};
use serde::{Deserialize, Serialize};

use crate::cli::Exit;
use crate::failure::Failure;
use crate::ports::Disk;

#[derive(Debug)]
pub struct Plan {
    workspace: PathBuf,
    diff: PathBuf,
    changed: PathBuf,
    reused: PathBuf,
    caught: PathBuf,
    output: PathBuf,
    prior: Option<Prior>,
}

#[derive(Debug)]
pub(crate) struct Prior {
    numstat: PathBuf,
    outcomes: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
struct Mutant {
    name: String,
    file: String,
}

#[derive(Debug, Deserialize, Serialize)]
enum Scenario {
    Baseline,
    Mutant(Mutant),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum Summary {
    CaughtMutant,
    Unviable,
    #[serde(other)]
    Unproven,
}

#[derive(Debug, Deserialize, Serialize)]
struct Outcome {
    scenario: Scenario,
    summary: Summary,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Outcomes {
    outcomes: Vec<Outcome>,
}

#[derive(Debug)]
enum Reusable {
    Nothing(Reason),
    Caught { names: BTreeSet<String>, changed: BTreeSet<PathBuf> },
}

#[derive(Debug)]
enum Reason {
    NoEarlierRun,
    TestsLost(PathBuf),
}

#[derive(Debug)]
struct Row {
    deleted: String,
    path: PathBuf,
}

#[derive(Debug)]
struct Shards(usize);

impl Plan {
    const PER_SHARD: usize = 120;

    pub(crate) fn new([workspace, diff, changed, reused, caught, output]: [PathBuf; 6], prior: Option<Prior>) -> Plan {
        Plan { workspace, diff, changed, reused, caught, output, prior }
    }

    pub(crate) fn run(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let workspace = Mutant::listed(&self.workspace)?;
        let in_diff: BTreeSet<String> = Mutant::listed(&self.diff)?.into_iter().map(|mutant| mutant.name).collect();
        let tested = Row::tested(Row::read(&Disk::read(&self.changed)?)?);
        let reusable = self.prior.as_ref().map_or(Ok(Reusable::Nothing(Reason::NoEarlierRun)), Prior::reusable)?;
        let count = workspace.len();
        let (selected, skipped): (Vec<Mutant>, Vec<Mutant>) =
            workspace.into_iter().partition(|mutant| mutant.selected(&in_diff, &tested).is_some());
        let chosen = selected.len();
        let (reused, untested): (Vec<Mutant>, Vec<Mutant>) =
            selected.into_iter().partition(|mutant| reusable.reuses(mutant).is_some());
        let shards = Shards(untested.len().div_ceil(Plan::PER_SHARD).max(1));
        let names: String = skipped.iter().chain(&reused).map(|mutant| [mutant.name.as_str(), "\n"].concat()).collect();
        let outcomes = Outcomes {
            outcomes: reused
                .into_iter()
                .map(|mutant| Outcome { scenario: Scenario::Mutant(mutant), summary: Summary::CaughtMutant })
                .collect(),
        };
        Disk::write(&self.reused, &serde_json::to_string(&outcomes)?)?;
        Disk::write(&self.caught, &names)?;
        Disk::append(&self.output, &format!("shards={shards}\n"))?;
        let crates: Vec<String> = tested.iter().map(|path| path.display().to_string()).collect();
        writeln!(out, "{reusable}")?;
        writeln!(out, "mutants: every one in the crates whose tests changed: [{}]", crates.join(", "))?;
        writeln!(
            out,
            "mutants: {chosen} of {count} selected, {} in the diff; {} reused, {} to test in {} shards",
            in_diff.len(),
            outcomes.outcomes.len(),
            untested.len(),
            shards.0
        )?;
        Ok(Exit::Passed)
    }
}

impl Mutant {
    fn listed(path: &Path) -> Result<Vec<Mutant>, Failure> {
        let text = Disk::read(path)?;
        Ok(serde_json::Deserializer::from_str(&text).into_iter().next().unwrap_or_else(|| Ok(Vec::new()))?)
    }

    fn selected(&self, in_diff: &BTreeSet<String>, tested: &BTreeSet<PathBuf>) -> Option<()> {
        let file = Path::new(&self.file);
        in_diff.get(&self.name).map(|_| ()).or(tested.iter().find(|root| file.starts_with(root)).map(|_| ()))
    }
}

impl Prior {
    const TESTS: &'static str = "tests";
    const UNTOUCHED: &'static str = "0";

    pub(crate) fn given(arguments: &[OsString]) -> Option<Prior> {
        match arguments {
            [numstat, outcomes @ ..] => {
                Some(Prior { numstat: PathBuf::from(numstat), outcomes: outcomes.iter().map(PathBuf::from).collect() })
            }
            [] => None,
        }
    }

    fn reusable(&self) -> Result<Reusable, Failure> {
        let rows = Row::read(&Disk::read(&self.numstat)?)?;
        let tests = Component::Normal(Prior::TESTS.as_ref());
        let lost = rows
            .iter()
            .find(|row| row.deleted != Prior::UNTOUCHED && row.path.components().any(|part| part == tests))
            .map(|row| row.path.clone());
        lost.map_or_else(|| self.caught(rows), |path| Ok(Reusable::Nothing(Reason::TestsLost(path))))
    }

    fn caught(&self, rows: Vec<Row>) -> Result<Reusable, Failure> {
        let outcomes: Vec<Outcomes> = self
            .outcomes
            .iter()
            .map(|path| Ok(serde_json::from_str(&Disk::read(path)?)?))
            .collect::<Result<_, Failure>>()?;
        let names = outcomes.into_iter().flat_map(|run| run.outcomes).filter_map(Outcome::caught).collect();
        Ok(Reusable::Caught { names, changed: rows.into_iter().map(|row| row.path).collect() })
    }
}

impl Outcome {
    fn caught(self) -> Option<String> {
        match (self.scenario, self.summary) {
            (Scenario::Mutant(mutant), Summary::CaughtMutant | Summary::Unviable) => Some(mutant.name),
            (Scenario::Mutant(_), Summary::Unproven) | (Scenario::Baseline, _) => None,
        }
    }
}

impl Reusable {
    fn reuses(&self, mutant: &Mutant) -> Option<()> {
        match self {
            Reusable::Nothing(_) => None,
            Reusable::Caught { names, changed } => {
                names.get(&mutant.name).filter(|_| !changed.contains(Path::new(&mutant.file))).map(|_| ())
            }
        }
    }
}

impl Row {
    fn tested(rows: Vec<Row>) -> BTreeSet<PathBuf> {
        let tests = Component::Normal(Prior::TESTS.as_ref());
        rows.into_iter()
            .filter(|row| row.path.components().any(|part| part == tests))
            .map(|row| row.path.components().take_while(|part| *part != tests).collect())
            .collect()
    }

    fn read(numstat: &str) -> Result<Vec<Row>, Failure> {
        numstat
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .map(|line| match line.parse::<Fields>().as_ref().map(AsRef::as_ref) {
                Ok([_, deleted, path]) => Ok(Row { deleted: deleted.clone(), path: PathBuf::from(path) }),
                Ok(_) | Err(_) => Err(Failure::Numstat(line.clone())),
            })
            .collect()
    }
}

impl fmt::Display for Reusable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reusable::Nothing(Reason::NoEarlierRun) => formatter.write_str("mutants: no earlier run to reuse"),
            Reusable::Nothing(Reason::TestsLost(path)) => {
                write!(formatter, "mutants: reusing nothing, since test code lost lines in {}", path.display())
            }
            Reusable::Caught { names, changed } => write!(
                formatter,
                "mutants: reusing the earlier run's {} caught, outside the {} files changed since",
                names.len(),
                changed.len()
            ),
        }
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indices: Vec<String> = (0..self.0).map(|index| index.to_string()).collect();
        write!(formatter, "[{}]", indices.join(","))
    }
}

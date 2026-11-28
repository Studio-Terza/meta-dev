use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use console_ports::Answer;
use grammar::lines::{Fields, Lines};
use pipeline::{Measure, Pipeline};
use serde::Deserialize;

use crate::failure::Failure;
use crate::ports::Disk;

#[derive(Debug)]
pub struct Plan {
    workspace: PathBuf,
    diff: PathBuf,
    changed: PathBuf,
    unselected: PathBuf,
    output: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct Mutant {
    name: String,
    file: String,
}

#[derive(Debug)]
struct Selection {
    in_diff: BTreeSet<String>,
    tested: BTreeSet<PathBuf>,
}

#[derive(Debug, Measure)]
struct Partition {
    selected: Vec<Mutant>,
    unselected: Vec<Mutant>,
}

#[derive(Debug)]
struct Sharded {
    selected: usize,
    shards: Shards,
    unselected: Vec<Mutant>,
}

#[derive(Debug)]
struct Planned {
    unselected: String,
    output: String,
    shown: String,
}

#[derive(Debug)]
struct Shards(usize);

impl Plan {
    const PER_SHARD: usize = 120;

    pub(crate) fn new([workspace, diff, changed, unselected, output]: [PathBuf; 5]) -> Plan {
        Plan { workspace, diff, changed, unselected, output }
    }

    pub(crate) fn answered(&self) -> Result<Answer, Failure> {
        let workspace = Mutant::listed(&self.workspace)?;
        let in_diff = Mutant::listed(&self.diff)?.into_iter().map(|mutant| mutant.name).collect();
        let selection = Selection { in_diff, tested: Changed::tested(&Disk::read(&self.changed)?)? };
        let planned = selection.run(workspace);
        Disk::write(&self.unselected, &planned.unselected)?;
        Disk::append(&self.output, &planned.output)?;
        Ok(Answer::new(ExitCode::SUCCESS).showing(planned.shown))
    }
}

impl Pipeline for Selection {
    type Unit = Mutant;
    type Parsed = Partition;
    type Measured = Sharded;
    type Act = Planned;

    fn parsed(&self, mutant: Mutant) -> Partition {
        let file = Path::new(&mutant.file);
        let named = self.in_diff.get(&mutant.name).map(|_| ());
        let chosen = named.or(self.tested.iter().find(|root| file.starts_with(root)).map(|_| ()));
        match chosen {
            Some(()) => Partition { selected: vec![mutant], unselected: Vec::new() },
            None => Partition { selected: Vec::new(), unselected: vec![mutant] },
        }
    }

    fn measured(&self, partition: Partition) -> Sharded {
        let selected = partition.selected.len();
        let shards = Shards(selected.div_ceil(Plan::PER_SHARD).max(1));
        Sharded { selected, shards, unselected: partition.unselected }
    }

    fn act(&self, sharded: Sharded) -> Planned {
        let Sharded { selected, shards, unselected } = sharded;
        let count = selected + unselected.len();
        let crates: Vec<String> = self.tested.iter().map(|path| path.display().to_string()).collect();
        Planned {
            unselected: unselected.iter().map(|mutant| [mutant.name.as_str(), "\n"].concat()).collect(),
            output: format!("shards={shards}\n"),
            shown: format!(
                "mutants: every one in the crates whose tests changed: [{}]\nmutants: {selected} of {count} selected, {} in the diff; every one tested, in {} shards\n",
                crates.join(", "),
                self.in_diff.len(),
                shards.0
            ),
        }
    }
}

impl Mutant {
    fn listed(path: &Path) -> Result<Vec<Mutant>, Failure> {
        let text = Disk::read(path)?;
        Ok(serde_json::Deserializer::from_str(&text).into_iter().next().unwrap_or_else(|| Ok(Vec::new()))?)
    }
}

#[derive(Debug)]
struct Changed;

impl Changed {
    const TESTS: &'static str = "tests";

    fn tested(numstat: &str) -> Result<BTreeSet<PathBuf>, Failure> {
        let tests = Component::Normal(Changed::TESTS.as_ref());
        let paths: Vec<PathBuf> = Lines::from(numstat)
            .into_iter()
            .map(|line| match Fields::from(line.as_str()).as_ref() {
                [_, _, path] => Ok(PathBuf::from(path)),
                _ => Err(Failure::Numstat(line)),
            })
            .collect::<Result<_, Failure>>()?;
        Ok(paths
            .into_iter()
            .filter(|path| path.components().any(|part| part == tests))
            .map(|path| path.components().take_while(|part| *part != tests).collect())
            .collect())
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indices: Vec<String> = (0..self.0).map(|index| index.to_string()).collect();
        write!(formatter, "[{}]", indices.join(","))
    }
}

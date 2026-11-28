use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use console_ports::Answer;
use grammar::lines::{Fields, Lines};
use pipeline::Measure;
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

#[derive(Debug, Measure)]
struct Partition {
    selected: Vec<Mutant>,
    unselected: Vec<Mutant>,
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
        let in_diff: BTreeSet<String> = Mutant::listed(&self.diff)?.into_iter().map(|mutant| mutant.name).collect();
        let tested = Changed::tested(&Disk::read(&self.changed)?)?;
        let count = workspace.len();
        let Partition { selected, unselected } =
            Partition::total(workspace.into_iter().map(|mutant| mutant.parted(&in_diff, &tested)));
        let shards = Shards(selected.len().div_ceil(Plan::PER_SHARD).max(1));
        let names: String = unselected.iter().map(|mutant| [mutant.name.as_str(), "\n"].concat()).collect();
        Disk::write(&self.unselected, &names)?;
        Disk::append(&self.output, &format!("shards={shards}\n"))?;
        let crates: Vec<String> = tested.iter().map(|path| path.display().to_string()).collect();
        Ok(Answer::new(ExitCode::SUCCESS).showing(format_args!(
            "mutants: every one in the crates whose tests changed: [{}]\nmutants: {} of {count} selected, {} in the diff; every one tested, in {} shards\n",
            crates.join(", "),
            selected.len(),
            in_diff.len(),
            shards.0
        )))
    }
}

impl Mutant {
    fn listed(path: &Path) -> Result<Vec<Mutant>, Failure> {
        let text = Disk::read(path)?;
        Ok(serde_json::Deserializer::from_str(&text).into_iter().next().unwrap_or_else(|| Ok(Vec::new()))?)
    }

    fn parted(self, in_diff: &BTreeSet<String>, tested: &BTreeSet<PathBuf>) -> Partition {
        let file = Path::new(&self.file);
        let named = in_diff.get(&self.name).map(|_| ());
        match named.or(tested.iter().find(|root| file.starts_with(root)).map(|_| ())) {
            Some(()) => Partition { selected: vec![self], unselected: Vec::new() },
            None => Partition { selected: Vec::new(), unselected: vec![self] },
        }
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

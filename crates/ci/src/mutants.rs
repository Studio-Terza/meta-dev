use std::collections::BTreeSet;
use std::fmt;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use grammar_lines::{Fields, Lines};
use serde::Deserialize;

use crate::cli::Exit;
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
struct Shards(usize);

impl Plan {
    const PER_SHARD: usize = 120;

    pub(crate) fn new([workspace, diff, changed, unselected, output]: [PathBuf; 5]) -> Plan {
        Plan { workspace, diff, changed, unselected, output }
    }

    pub(crate) fn run(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let workspace = Mutant::listed(&self.workspace)?;
        let in_diff: BTreeSet<String> = Mutant::listed(&self.diff)?.into_iter().map(|mutant| mutant.name).collect();
        let tested = Changed::tested(&Disk::read(&self.changed)?)?;
        let count = workspace.len();
        let (selected, unselected): (Vec<Mutant>, Vec<Mutant>) =
            workspace.into_iter().partition(|mutant| mutant.selected(&in_diff, &tested).is_some());
        let shards = Shards(selected.len().div_ceil(Plan::PER_SHARD).max(1));
        let names: String = unselected.iter().map(|mutant| [mutant.name.as_str(), "\n"].concat()).collect();
        Disk::write(&self.unselected, &names)?;
        Disk::append(&self.output, &format!("shards={shards}\n"))?;
        let crates: Vec<String> = tested.iter().map(|path| path.display().to_string()).collect();
        writeln!(out, "mutants: every one in the crates whose tests changed: [{}]", crates.join(", "))?;
        writeln!(
            out,
            "mutants: {} of {count} selected, {} in the diff; every one tested, in {} shards",
            selected.len(),
            in_diff.len(),
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

#[derive(Debug)]
struct Changed;

impl Changed {
    const TESTS: &'static str = "tests";

    fn tested(numstat: &str) -> Result<BTreeSet<PathBuf>, Failure> {
        let tests = Component::Normal(Changed::TESTS.as_ref());
        let paths: Vec<PathBuf> = numstat
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .map(|line| match line.parse::<Fields>().as_ref().map(AsRef::as_ref) {
                Ok([_, _, path]) => Ok(PathBuf::from(path)),
                Ok(_) | Err(_) => Err(Failure::Numstat(line.clone())),
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

mod boundary;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::boundary::MutantName;
use crate::cli::Exit;
use crate::failure::Failure;
use crate::ports::Disk;
use boundary::{Changed, Shards};

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
    name: MutantName,
    file: PathBuf,
}

impl Plan {
    const PER_SHARD: usize = 120;

    pub(crate) fn new([workspace, diff, changed, unselected, output]: [PathBuf; 5]) -> Plan {
        Plan { workspace, diff, changed, unselected, output }
    }

    pub(crate) fn run(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let workspace = Mutant::listed(&self.workspace)?;
        let in_diff: BTreeSet<MutantName> = Mutant::listed(&self.diff)?.into_iter().map(|mutant| mutant.name).collect();
        let tested = Changed::tested(&Disk::read(&self.changed)?)?;
        let count = workspace.len();
        let (selected, unselected): (Vec<Mutant>, Vec<Mutant>) =
            workspace.into_iter().partition(|mutant| mutant.selected(&in_diff, &tested).is_some());
        let shards = Shards::sharing(selected.len(), Plan::PER_SHARD);
        let names: String = unselected.iter().map(|mutant| mutant.name.line()).collect();
        Disk::write(&self.unselected, &names)?;
        Disk::append(&self.output, &format!("shards={shards}\n"))?;
        let crates: Vec<String> = tested.iter().map(|path| path.display().to_string()).collect();
        writeln!(out, "mutants: every one in the crates whose tests changed: [{}]", crates.join(", "))?;
        writeln!(
            out,
            "mutants: {} of {count} selected, {} in the diff; every one tested, in {} shards",
            selected.len(),
            in_diff.len(),
            shards.count()
        )?;
        Ok(Exit::Passed)
    }
}

impl Mutant {
    fn listed(path: &Path) -> Result<Vec<Mutant>, Failure> {
        let text = Disk::read(path)?;
        Ok(serde_json::Deserializer::from_str(&text).into_iter().next().unwrap_or_else(|| Ok(Vec::new()))?)
    }

    fn selected(&self, in_diff: &BTreeSet<MutantName>, tested: &BTreeSet<PathBuf>) -> Option<()> {
        in_diff.get(&self.name).map(|_| ()).or(tested.iter().find(|root| self.file.starts_with(root)).map(|_| ()))
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use json_ports::Json;

use crate::failure::Failure;
use crate::kills::log::{Kind, Test};
use crate::kills::outcomes::{Fate, Mutant, Ran};

#[derive(Debug, Default)]
pub(crate) struct Matrix {
    kills: BTreeMap<Test, BTreeSet<Mutant>>,
    caught: BTreeSet<Mutant>,
    missed: BTreeSet<Mutant>,
    unviable: BTreeSet<Mutant>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Report {
    caught: usize,
    unviable: usize,
    missed: Vec<Mutant>,
    unattributed: Vec<Mutant>,
    golden: Vec<Test>,
    tautological: Vec<Test>,
    redundant: Vec<Vec<Test>>,
    pinning: Vec<Test>,
    kills: Vec<Row>,
}

#[derive(Debug, serde::Serialize)]
struct Row {
    test: Test,
    killed: Vec<Mutant>,
}

impl Matrix {
    pub(crate) fn with(self, ran: &Ran) -> Matrix {
        let seen = ran.log().tests().fold(self, Matrix::seen);
        match ran.fate() {
            Fate::Inventory => seen,
            Fate::Caught(mutant) => {
                ran.log().failed().fold(seen, |matrix, test| matrix.killed(test, mutant)).caught(mutant)
            }
            Fate::Missed(mutant) => seen.missed(mutant),
            Fate::Unviable(mutant) => seen.unviable(mutant),
        }
    }

    fn seen(mut self, test: &Test) -> Matrix {
        self.kills.entry(test.clone()).or_default();
        self
    }

    fn killed(mut self, test: &Test, mutant: &Mutant) -> Matrix {
        self.kills.entry(test.clone()).or_default().insert(mutant.clone());
        self
    }

    fn caught(mut self, mutant: &Mutant) -> Matrix {
        self.caught.insert(mutant.clone());
        self
    }

    fn missed(mut self, mutant: &Mutant) -> Matrix {
        self.missed.insert(mutant.clone());
        self
    }

    fn unviable(mut self, mutant: &Mutant) -> Matrix {
        self.unviable.insert(mutant.clone());
        self
    }
}

impl From<Matrix> for Report {
    fn from(matrix: Matrix) -> Report {
        let attributed: BTreeSet<&Mutant> = matrix.kills.values().flatten().collect();
        let golden: Vec<&Test> = matrix.kills.keys().filter(|test| test.kind() == Kind::Golden).collect();
        let noticed: BTreeSet<&Mutant> = golden.iter().filter_map(|test| matrix.kills.get(*test)).flatten().collect();
        let twins = matrix.kills.iter().filter(|(_, killed)| !killed.is_empty()).fold(
            BTreeMap::<&BTreeSet<Mutant>, Vec<Test>>::new(),
            |mut twins, (test, killed)| {
                twins.entry(killed).or_default().push(test.clone());
                twins
            },
        );
        Report {
            caught: matrix.caught.len(),
            unviable: matrix.unviable.len(),
            missed: matrix.missed.iter().cloned().collect(),
            unattributed: matrix.caught.iter().filter(|mutant| !attributed.contains(mutant)).cloned().collect(),
            golden: golden.into_iter().cloned().collect(),
            tautological: matrix
                .kills
                .iter()
                .filter(|(_, killed)| killed.is_empty())
                .map(|(test, _)| test.clone())
                .collect(),
            redundant: twins.into_values().filter(|tests| tests.len() > 1).collect(),
            pinning: matrix
                .kills
                .iter()
                .filter(|(test, killed)| {
                    test.kind() == Kind::Other
                        && !killed.is_empty()
                        && killed.iter().all(|mutant| !noticed.contains(mutant))
                })
                .map(|(test, _)| test.clone())
                .collect(),
            kills: matrix
                .kills
                .iter()
                .map(|(test, killed)| Row { test: test.clone(), killed: killed.iter().cloned().collect() })
                .collect(),
        }
    }
}

impl Report {
    pub(crate) fn json(&self) -> Result<String, Failure> {
        Json::pretty(self).map(|json| [json.as_str(), "\n"].concat()).map_err(Failure::Unserialized)
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "kill matrix: {} mutants caught, {} by no named test, {} missed, {} unviable; {} tests, {} golden: {} tautological, {} in {} redundant groups, {} pinning",
            self.caught,
            self.unattributed.len(),
            self.missed.len(),
            self.unviable,
            self.kills.len(),
            self.golden.len(),
            self.tautological.len(),
            self.redundant.iter().map(Vec::len).sum::<usize>(),
            self.redundant.len(),
            self.pinning.len(),
        )
    }
}

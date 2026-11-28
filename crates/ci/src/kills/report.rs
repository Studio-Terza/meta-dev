use std::collections::BTreeSet;
use std::fmt;

use pipeline::{Edges, Measure};

use crate::kills::log::{Kind, Test};
use crate::kills::outcomes::{Fate, Mutant, Ran};

#[derive(Debug, Measure)]
pub(crate) struct Matrix {
    kills: Edges<Test, Mutant>,
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
    pub(crate) fn of(ran: &Ran) -> Matrix {
        let seen = Edges::total(ran.log().tests().cloned().map(Edges::node));
        let fated = |mutant: &Mutant| BTreeSet::from([mutant.clone()]);
        match ran.fate() {
            Fate::Inventory => Matrix { kills: seen, ..Matrix::default() },
            Fate::Caught(mutant) => {
                let killed = ran.log().failed().map(|test| Edges::edge(test.clone(), mutant.clone()));
                Matrix { kills: seen.joined(Edges::total(killed)), caught: fated(mutant), ..Matrix::default() }
            }
            Fate::Missed(mutant) => Matrix { kills: seen, missed: fated(mutant), ..Matrix::default() },
            Fate::Unviable(mutant) => Matrix { kills: seen, unviable: fated(mutant), ..Matrix::default() },
        }
    }
}

impl From<Matrix> for Report {
    fn from(matrix: Matrix) -> Report {
        let kills = &matrix.kills;
        let attributed = kills.reached();
        let golden: Vec<&Test> = kills.nodes().filter(|test| test.kind() == Kind::Golden).collect();
        let noticed: BTreeSet<Mutant> =
            golden.iter().filter_map(|test| kills.targets(test)).flatten().cloned().collect();
        let killed = |test: &Test| kills.targets(test).cloned().unwrap_or_default();
        let twins = kills.neighbourhoods().crowded();
        Report {
            caught: matrix.caught.len(),
            unviable: matrix.unviable.len(),
            missed: matrix.missed.iter().cloned().collect(),
            unattributed: matrix.caught.iter().filter(|mutant| !attributed.contains(mutant)).cloned().collect(),
            golden: golden.into_iter().cloned().collect(),
            tautological: kills.bare().cloned().collect(),
            redundant: twins
                .nodes()
                .filter(|shared| !shared.is_empty())
                .filter_map(|shared| twins.targets(shared))
                .map(|tests| tests.iter().cloned().collect())
                .collect(),
            pinning: kills
                .nodes()
                .filter(|test| test.kind() == Kind::Other)
                .filter(|test| killed(test).first().is_some() && killed(test).is_disjoint(&noticed))
                .cloned()
                .collect(),
            kills: kills
                .nodes()
                .map(|test| Row { test: test.clone(), killed: killed(test).into_iter().collect() })
                .collect(),
        }
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

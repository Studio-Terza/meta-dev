mod crate_budget;

use std::collections::BTreeMap;
use std::fmt;
use std::iter;

use graph::Model;
use lint_core::{Parsed, SourceFile};
use lint_file::Gleaning;
use lint_placement::CrateName;
use lint_process_starters::{CommandPaths, Scope};

use crate_budget::{BudgetGathering, CrateBudget};
use lint_sweep::{Gathers, Sweep, Watching};

#[derive(Clone, Debug, Default)]
pub struct TestBudget(BTreeMap<CrateName, CrateBudget>);

impl TestBudget {
    #[must_use]
    pub fn measured(files: &[SourceFile]) -> TestBudget {
        TestBudget(files.iter().fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.head().placement().package().root());
            let budget: CrateBudget = crates.remove(&name).unwrap_or_default();
            crates.insert(name, budget + Sweep::gathered(file, BudgetGathering::from(file)));
            crates
        }))
    }

    #[must_use]
    pub fn started(&self, model: Option<&Model>) -> TestBudget {
        let starts = model.map_or_default(|found| CommandPaths::queried(found, Scope::Everywhere));
        let parts = starts
            .iter()
            .map(|placed| TestBudget(BTreeMap::from([(CrateName::from(placed.krate()), CrateBudget::start())])));
        iter::once(self.clone()).chain(parts).collect()
    }

    #[must_use]
    pub fn of_crate(&self, name: &CrateName) -> Option<CrateBudget> {
        self.0.get(name).copied()
    }
}

impl fmt::Display for TestBudget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "test budget, report only:")?;
        self.0.iter().try_for_each(|(name, budget)| writeln!(formatter, "{name}: {budget}"))
    }
}

impl FromIterator<TestBudget> for TestBudget {
    fn from_iter<Parts: IntoIterator<Item = TestBudget>>(parts: Parts) -> TestBudget {
        TestBudget(parts.into_iter().flat_map(|part| part.0).fold(BTreeMap::new(), |mut crates, (name, budget)| {
            let held: CrateBudget = crates.remove(&name).unwrap_or_default();
            crates.insert(name, held + budget);
            crates
        }))
    }
}

impl Gleaning for TestBudget {
    type Part<'head> = TestBudget;
    type Gleaned<'head> = TestBudget;

    type Gleaner<'head> = Gleaner;

    fn gleaner(parsed: Parsed<'_, '_>) -> Gleaner {
        let file = parsed.file();
        Gleaner { name: CrateName::from(file.head().placement().package().root()), budget: BudgetGathering::from(file) }
    }
}

#[derive(Debug)]
pub struct Gleaner {
    name: CrateName,
    budget: BudgetGathering,
}

impl Gathers<TestBudget> for Gleaner {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.budget.watching()
    }

    fn gathered(self) -> TestBudget {
        TestBudget(BTreeMap::from([(self.name, self.budget.gathered())]))
    }
}

mod crate_budget;

use std::collections::BTreeMap;
use std::fmt;

use lint_core::SourceFile;
use lint_placement::CrateName;
use lint_process_starters::{CommandPaths, Scope};

use crate_budget::{BudgetGathering, CrateBudget};
use lint_sweep::Sweep;

#[derive(Clone, Debug, Default)]
pub struct TestBudget(BTreeMap<CrateName, CrateBudget>);

impl TestBudget {
    #[must_use]
    pub fn measured(files: &[SourceFile]) -> TestBudget {
        TestBudget(files.iter().fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.head().placement().package().root());
            let budget: CrateBudget = crates.remove(&name).unwrap_or_default();
            let starts = CommandPaths::found(file, Scope::Everywhere)
                .iter()
                .map(|_| CrateBudget::start())
                .fold(Sweep::gathered(file, BudgetGathering::from(file)), |gathered, start| gathered + start);
            crates.insert(name, budget + starts);
            crates
        }))
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

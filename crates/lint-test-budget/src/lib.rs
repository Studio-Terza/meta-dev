mod crate_budget;

use std::collections::BTreeMap;
use std::fmt;

use code_model::Model;
use lint_core::SourceFile;
use lint_placement::CrateName;

use crate_budget::{CrateBudget, Starts};

#[derive(Debug, Default)]
pub struct TestBudget(BTreeMap<CrateName, CrateBudget>);

impl TestBudget {
    #[must_use]
    pub fn measured(files: &[SourceFile], model: &Model) -> TestBudget {
        let starts = Starts::counted(model);
        TestBudget(files.iter().fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.placement().package().root());
            let budget: CrateBudget = crates.remove(&name).unwrap_or_default();
            crates.insert(name, budget + CrateBudget::of_file(file, &starts));
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

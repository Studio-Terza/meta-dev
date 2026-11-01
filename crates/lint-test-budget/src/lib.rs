mod crate_budget;

use std::collections::BTreeMap;
use std::fmt;

use lint_core::SourceFile;
use lint_placement::CrateName;

use crate_budget::CrateBudget;

#[derive(Debug, Default)]
pub struct TestBudget(BTreeMap<CrateName, CrateBudget>);

impl TestBudget {
    #[must_use]
    pub fn measured(files: &[SourceFile]) -> TestBudget {
        TestBudget(files.iter().fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.placement().package().root());
            let budget: CrateBudget = crates.remove(&name).unwrap_or_default();
            crates.insert(name, budget + CrateBudget::from(file));
            crates
        }))
    }

    #[must_use]
    pub fn of_crate(&self, name: &CrateName) -> Option<String> {
        self.0.get(name).map(ToString::to_string)
    }
}

impl fmt::Display for TestBudget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "test budget, report only:")?;
        self.0.iter().try_for_each(|(name, budget)| writeln!(formatter, "{name}: {budget}"))
    }
}

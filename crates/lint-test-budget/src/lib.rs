mod crate_budget;

use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Parsed, SourceFile};
use lint_file::Gleaning;
use lint_placement::CrateName;

use crate_budget::CrateBudget;

#[derive(Debug, Default)]
pub struct TestBudget(BTreeMap<CrateName, CrateBudget>);

impl TestBudget {
    #[must_use]
    pub fn measured(files: &[SourceFile]) -> TestBudget {
        TestBudget(files.iter().fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.head().placement().package().root());
            let budget: CrateBudget = crates.remove(&name).unwrap_or_default();
            crates.insert(name, budget + CrateBudget::from(file));
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

    fn gleaned(parsed: Parsed<'_, '_>) -> TestBudget {
        let file = parsed.file();
        TestBudget(BTreeMap::from([(
            CrateName::from(file.head().placement().package().root()),
            CrateBudget::from(file),
        )]))
    }
}

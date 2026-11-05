mod lost;

use std::collections::BTreeMap;

use lint_placement::CrateName;
use test_facts::{Claims, Test, Tests};

pub(crate) use lost::Lost;

#[derive(Debug)]
pub(crate) struct Pairing<'tests, 'file> {
    before: &'tests Tests<'file>,
    after: BTreeMap<&'tests CrateName, Claims>,
}

impl<'tests, 'file> Pairing<'tests, 'file> {
    pub(crate) fn across(before: &'tests Tests<'file>, after: &'tests Tests<'file>) -> Pairing<'tests, 'file> {
        let grouped = after.all().fold(BTreeMap::<&CrateName, Vec<&Test<'file>>>::new(), |mut grouped, test| {
            grouped.entry(test.package()).or_default().push(test);
            grouped
        });
        Pairing {
            before,
            after: grouped
                .into_iter()
                .map(|(package, tests)| (package, tests.iter().flat_map(|test| test.claims()).collect()))
                .collect(),
        }
    }

    pub(crate) fn lost(&self) -> Vec<Lost<'tests, 'file>> {
        let empty = Claims::default();
        self.before
            .all()
            .flat_map(|test| Lost::in_package(test, self.after.get(test.package()).unwrap_or(&empty)))
            .collect()
    }
}

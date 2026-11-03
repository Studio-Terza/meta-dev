use std::collections::BTreeMap;

use crate::Pair;
use crate::declared::Declared;
use crate::owner::Owner;
use lint_core::Violation;

#[derive(Debug)]
pub struct Pairs;

impl Pairs {
    pub fn listed(declared: &[Declared]) -> Vec<Violation> {
        let grouped = declared.iter().fold(BTreeMap::<&Owner, Vec<&Declared>>::new(), |mut groups, method| {
            groups.entry(method.owner()).or_default().push(method);
            groups
        });
        grouped.values().flat_map(|methods| Pair::ALL.iter().flat_map(|pair| pair.violations(methods))).collect()
    }
}

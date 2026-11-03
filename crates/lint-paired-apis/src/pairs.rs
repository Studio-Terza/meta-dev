use std::collections::BTreeMap;

use crate::Pair;
use crate::declared::Declared;
use crate::owner::Owner;
use lint_core::Violation;

#[derive(Debug)]
pub struct Pairs;

impl Pairs {
    pub fn listed<'file>(declared: &[Declared<'file>]) -> Vec<Violation<'file>> {
        let grouped = declared.iter().fold(BTreeMap::<&Owner, Vec<&Declared<'file>>>::new(), |mut groups, method| {
            groups.entry(method.owner()).or_default().push(method);
            groups
        });
        grouped.values().flat_map(|methods| Pair::ALL.iter().flat_map(|pair| pair.violations(methods))).collect()
    }
}

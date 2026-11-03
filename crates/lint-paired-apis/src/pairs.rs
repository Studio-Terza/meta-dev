use std::collections::BTreeMap;

use crate::Pair;
use crate::declared::Declared;
use crate::owner::Owner;
use lint_core::Violation;

#[derive(Debug)]
pub struct Pairs;

impl Pairs {
    pub fn listed<'file>(declared: &BTreeMap<Owner<'file>, Vec<Declared<'file>>>) -> Vec<Violation<'file>> {
        declared.values().flat_map(|methods| Pair::ALL.iter().flat_map(|pair| pair.violations(methods))).collect()
    }
}

mod pair;

use crate::declared::Declared;
use lint_core::Violation;
use pair::Pair;

#[derive(Debug)]
pub struct Pairs;

impl Pairs {
    pub fn listed<'file, 'owned>(declared: impl Iterator<Item = &'owned Vec<Declared<'file>>>) -> Vec<Violation<'file>>
    where
        'file: 'owned,
    {
        declared.flat_map(|methods| Pair::ALL.iter().flat_map(|pair| pair.violations(methods))).collect()
    }
}

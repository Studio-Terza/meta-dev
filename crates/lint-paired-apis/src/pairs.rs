use std::collections::{BTreeMap, BTreeSet};

use crate::declared::Declared;
use crate::owner::Owner;
use crate::{Finding, Pair};
use lint_core::Violation;
use lint_layout::Name;

#[derive(Debug)]
pub struct Pairs;

impl Pairs {
    pub fn listed<'file>(declared: &BTreeMap<Owner<'file>, Vec<Declared<'file>>>) -> Vec<Violation<'file>> {
        declared.values().flat_map(|methods| Self::among(methods)).collect()
    }

    fn among<'file>(methods: &[Declared<'file>]) -> Vec<Violation<'file>> {
        let names: BTreeSet<&Name> = methods.iter().map(Declared::method).collect();
        Pair::ALL
            .iter()
            .flat_map(|pair| methods.iter().filter_map(move |method| Some((method, pair.partner(method.method())?))))
            .filter(|(_, partner)| names.contains(partner))
            .map(|(method, partner)| Violation::new(method.location(), Finding::new(method.method(), partner)))
            .collect()
    }
}

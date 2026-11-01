use std::collections::{BTreeMap, BTreeSet};

use crate::declared::Declared;
use crate::owner::Owner;
use crate::{Finding, Pair};
use lint_core::Violation;
use lint_layout::Name;

#[derive(Debug)]
pub struct Pairs;

impl Pairs {
    pub fn listed(declared: &[Declared]) -> Vec<Violation> {
        let grouped = declared.iter().fold(BTreeMap::<&Owner, Vec<&Declared>>::new(), |mut groups, method| {
            groups.entry(method.owner()).or_default().push(method);
            groups
        });
        grouped.values().flat_map(|methods| Self::among(methods)).collect()
    }

    fn among(methods: &[&Declared]) -> Vec<Violation> {
        let names: BTreeSet<&Name> = methods.iter().map(|method| method.method()).collect();
        Pair::ALL
            .iter()
            .flat_map(|pair| methods.iter().filter_map(move |method| Some((*method, pair.partner(method.method())?))))
            .filter(|(_, partner)| names.contains(partner))
            .map(|(method, partner)| {
                Violation::new(method.location().clone(), Finding::new(method.method().clone(), partner))
            })
            .collect()
    }
}

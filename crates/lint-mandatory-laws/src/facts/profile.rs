use std::collections::BTreeSet;
use std::path::Path;

use super::obligation::Obligation;
use crate::need::Need;
use lint_core::Site;
use lint_layout::LineNumber;
use lint_layout::Name;
use syn::Ident;

#[derive(Debug)]
pub(crate) struct Profile {
    owner: Name,
    line: LineNumber,
    facts: BTreeSet<Name>,
}

impl Profile {
    const PAIRS: [(&'static str, &'static str); 3] =
        [("Display", "FromStr"), ("Serialize", "Deserialize"), ("encode", "decode")];
    const ORDERS: [&'static str; 2] = ["Ord", "PartialEq"];
    const TO: &'static str = "to";
    const FROM: &'static str = "from";

    pub(crate) fn declared(owner: &Ident, facts: Vec<Name>) -> Profile {
        Profile { owner: Name::from(owner), line: LineNumber::from(owner.span()), facts: facts.into_iter().collect() }
    }

    pub(crate) fn owner(&self) -> &Name {
        &self.owner
    }

    pub(crate) fn merged(mut self, other: Profile) -> Profile {
        self.facts.extend(other.facts);
        self
    }

    pub(crate) fn obligations(&self, path: &Path) -> Vec<Obligation> {
        let roundtrip = self.has_pair().then_some(Need::Roundtrip);
        let law = Self::ORDERS.iter().any(|order| self.knows(order)).then_some(Need::Law);
        roundtrip
            .into_iter()
            .chain(law)
            .map(|need| Obligation::new(self.owner.clone(), Site::new(path, self.line), need))
            .collect()
    }

    fn knows(&self, fact: &str) -> bool {
        self.facts.contains(&Name::from(fact))
    }

    fn forms(&self, lead: &str) -> BTreeSet<Vec<Name>> {
        self.facts
            .iter()
            .map(Name::words)
            .filter_map(|words| words.split_first().filter(|(first, _)| **first == lead).map(|(_, rest)| rest.to_vec()))
            .collect()
    }

    fn has_pair(&self) -> bool {
        Self::PAIRS.iter().any(|(left, right)| self.knows(left) && self.knows(right))
            || !self.forms(Self::TO).is_disjoint(&self.forms(Self::FROM))
    }
}

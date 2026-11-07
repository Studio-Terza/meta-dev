mod obligation;
mod pair;

use std::collections::{BTreeMap, BTreeSet};

use super::super::exposure::Exposure;
use crate::coverage::Coverage;
use crate::subject::Subject;
use lint_core::{Site, Violation};
use lint_layout::{Name, Word, words};
use obligation::Obligation;
use pair::Pair;

#[derive(Debug)]
pub(crate) struct Facets<'file> {
    site: Site<'file>,
    exposure: Exposure,
    facts: BTreeSet<Name>,
}

impl<'file> Facets<'file> {
    const TRAIT_PAIRS: [[Word<'static>; 2]; 2] = [words!["Display", "FromStr"], words!["Serialize", "Deserialize"]];
    const METHOD_PAIR: [Word<'static>; 2] = words!["encode", "decode"];
    const TO: Word<'static> = Word::new("to");
    const FROM: Word<'static> = Word::new("from");

    pub(crate) fn new(site: Site<'file>, exposure: Exposure, facts: Vec<Name>) -> Self {
        Facets { site, exposure, facts: facts.into_iter().collect() }
    }

    pub(crate) fn exposure(&self) -> Exposure {
        self.exposure
    }

    pub(crate) fn after(self, earlier: Option<Self>) -> Self {
        match earlier {
            Some(earlier) => earlier.merged(self),
            None => self,
        }
    }

    pub(crate) fn merged(mut self, other: Self) -> Self {
        self.facts.extend(other.facts);
        self.exposure = self.exposure.max(other.exposure);
        self
    }

    pub(crate) fn unmet<'facts>(
        &'facts self,
        subject: &'facts Subject<'file>,
        coverage: &Coverage<'_>,
    ) -> Vec<Violation<'file>> {
        let roundtrips = coverage.roundtrips_naming(subject);
        let methods = self.method_pairs();
        let spare = roundtrips
            .iter()
            .filter(|mentions| methods.iter().find_map(|pair| pair.witnessed_by(mentions)).is_none())
            .count();
        let unwitnessed =
            methods.iter().filter(|pair| roundtrips.iter().find_map(|mentions| pair.witnessed_by(mentions)).is_none());
        let unspared = Self::TRAIT_PAIRS
            .iter()
            .filter_map(|[left, right]| Some(Pair::new(self.known(*left)?, self.known(*right)?)))
            .skip(spare);
        unwitnessed
            .copied()
            .chain(unspared)
            .map(|pair| Violation::new(self.site, Obligation::new(subject, pair)))
            .collect()
    }

    fn known(&self, fact: Word<'_>) -> Option<&Name> {
        self.facts.get(&Name::from(fact))
    }

    fn method_pairs(&self) -> BTreeSet<Pair<'_>> {
        let [encode, decode] = Self::METHOD_PAIR;
        let coded = self.known(encode).zip(self.known(decode)).map(|(left, right)| Pair::new(left, right));
        let from = self.forms(Self::FROM);
        let formed =
            self.forms(Self::TO).into_iter().filter_map(|(rest, to)| from.get(&rest).map(|from| Pair::new(to, from)));
        coded.into_iter().chain(formed).collect()
    }

    fn forms(&self, lead: Word<'_>) -> BTreeMap<Vec<Name>, &Name> {
        self.facts
            .iter()
            .filter_map(|fact| {
                let words = fact.words();
                words.first().filter(|first| **first == Name::from(lead))?;
                Some((words.into_iter().skip(1).collect(), fact))
            })
            .collect()
    }
}

use std::collections::{BTreeMap, BTreeSet};

use super::exposure::Exposure;
use crate::coverage::Coverage;
use crate::need::Need;
use crate::obligation::Obligation;
use crate::pair::Pair;
use crate::subject::Subject;
use lint_core::Site;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Profile {
    subject: Subject,
    site: Site,
    exposure: Exposure,
    facts: BTreeSet<Name>,
}

impl Profile {
    const TRAIT_PAIRS: [(&'static str, &'static str); 2] = [("Display", "FromStr"), ("Serialize", "Deserialize")];
    const METHOD_PAIR: (&'static str, &'static str) = ("encode", "decode");
    const ORDERS: [&'static str; 2] = ["Ord", "PartialEq"];
    const TO: &'static str = "to";
    const FROM: &'static str = "from";

    pub(crate) fn declared(subject: Subject, site: Site, exposure: Exposure, facts: Vec<Name>) -> Profile {
        Profile { subject, site, exposure, facts: facts.into_iter().collect() }
    }

    pub(crate) fn subject(&self) -> &Subject {
        &self.subject
    }

    pub(crate) fn exposure(&self) -> Exposure {
        self.exposure
    }

    pub(crate) fn merged(mut self, other: Profile) -> Profile {
        self.facts.extend(other.facts);
        self.exposure = self.exposure.max(other.exposure);
        self
    }

    pub(crate) fn unmet(&self, coverage: &Coverage) -> Vec<Obligation> {
        let roundtrips = coverage.roundtrips_naming(&self.subject);
        let methods = self.method_pairs();
        let spare =
            roundtrips.iter().filter(|mentions| !methods.iter().any(|pair| pair.witnessed_by(mentions))).count();
        let unwitnessed = methods.iter().filter(|pair| !roundtrips.iter().any(|mentions| pair.witnessed_by(mentions)));
        let unspared = self.trait_pairs().into_iter().skip(spare);
        let law = Self::ORDERS.iter().any(|order| self.knows(order)).then_some(Need::Law);
        let unlawed = law.filter(|_| !coverage.names(&self.subject));
        unwitnessed
            .cloned()
            .chain(unspared)
            .map(Need::Roundtrip)
            .chain(unlawed)
            .map(|need| Obligation::new(self.subject.clone(), self.site.clone(), need))
            .collect()
    }

    fn knows(&self, fact: &str) -> bool {
        self.facts.contains(&Name::from(fact))
    }

    fn trait_pairs(&self) -> Vec<Pair> {
        Self::TRAIT_PAIRS
            .iter()
            .filter(|(left, right)| self.knows(left) && self.knows(right))
            .map(|(left, right)| Pair::new(Name::from(*left), Name::from(*right)))
            .collect()
    }

    fn method_pairs(&self) -> BTreeSet<Pair> {
        let (encode, decode) = Self::METHOD_PAIR;
        let coded =
            (self.knows(encode) && self.knows(decode)).then(|| Pair::new(Name::from(encode), Name::from(decode)));
        let from = self.forms(Self::FROM);
        let formed = self
            .forms(Self::TO)
            .into_iter()
            .filter_map(|(rest, to)| from.get(&rest).map(|from| Pair::new(to, from.clone())));
        coded.into_iter().chain(formed).collect()
    }

    fn forms(&self, lead: &str) -> BTreeMap<Vec<Name>, Name> {
        self.facts
            .iter()
            .filter_map(|fact| {
                let words = fact.words();
                words.split_first().filter(|(first, _)| **first == lead).map(|(_, rest)| (rest.to_vec(), fact.clone()))
            })
            .collect()
    }
}

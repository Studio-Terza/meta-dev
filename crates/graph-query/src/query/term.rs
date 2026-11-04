use std::convert::Infallible;
use std::str::FromStr;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Term(Name);

impl Term {
    const SHORTEST_STEM: usize = 3;
    const PLURAL: char = 's';

    #[must_use]
    pub fn found_in(&self, name: &Name) -> Option<&Term> {
        name.humps().map(|hump| Term::from(&hump)).find(|hump| hump == self).map(|_| self)
    }
}

impl From<&Name> for Term {
    fn from(name: &Name) -> Term {
        let letters: Vec<char> = name.to_string().chars().flat_map(char::to_lowercase).collect();
        let stem = letters
            .split_last()
            .filter(|(last, _)| **last == Self::PLURAL)
            .map(|(_, rest)| rest)
            .filter(|rest| rest.len() >= Self::SHORTEST_STEM)
            .unwrap_or(letters.as_slice());
        Term(Name::from(stem.iter().collect::<String>().as_str()))
    }
}

impl FromStr for Term {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Term, Infallible> {
        Ok(Term::from(&Name::from(text)))
    }
}

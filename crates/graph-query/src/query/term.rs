use std::convert::Infallible;
use std::str::FromStr;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Term(Name);

impl Term {
    const SHORTEST_STEM: usize = 3;
    const PLURAL: char = 's';

    #[must_use]
    pub fn found_in(&self, name: &Name) -> Option<&Self> {
        name.humps().map(|hump| Self::from(hump.as_ref())).find(|hump| hump == self).map(|_| self)
    }
}

impl From<&Name> for Term {
    fn from(name: &Name) -> Self {
        let letters: Vec<char> = name.to_string().chars().flat_map(char::to_lowercase).collect();
        let stem = letters
            .split_last()
            .filter(|(last, _)| **last == Self::PLURAL)
            .map(|(_, rest)| rest)
            .filter(|rest| rest.len() >= Self::SHORTEST_STEM)
            .unwrap_or(letters.as_slice());
        Self(Name::from(stem.iter().collect::<String>().as_str()))
    }
}

impl FromStr for Term {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Infallible> {
        Ok(Self::from(&Name::from(text)))
    }
}

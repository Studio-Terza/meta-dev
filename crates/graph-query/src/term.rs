use std::convert::Infallible;
use std::str::FromStr;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Term(Name);

impl Term {
    const SHORTEST_STEM: usize = 3;

    #[must_use]
    pub fn found_in(&self, name: &Name) -> Option<&Term> {
        name.humps().map(|hump| Term::from(&hump)).find(|hump| hump == self).map(|_| self)
    }
}

impl From<&Name> for Term {
    fn from(name: &Name) -> Term {
        let letters: Vec<char> = name.lowered().to_string().chars().collect();
        let stem = match letters.split_last() {
            Some(('s', rest)) if rest.len() >= Self::SHORTEST_STEM => rest,
            _ => letters.as_slice(),
        };
        Term(Name::from(stem.iter().collect::<String>().as_str()))
    }
}

impl FromStr for Term {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Term, Infallible> {
        Ok(Term::from(&Name::from(text)))
    }
}

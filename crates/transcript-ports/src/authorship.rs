use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Authorship {
    Owner,
    Lead,
}

#[derive(Debug, thiserror::Error)]
#[error("{0:?} names neither the owner nor the lead")]
pub struct Unvoiced(String);

impl fmt::Display for Authorship {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Authorship::Owner => formatter.write_str("owner"),
            Authorship::Lead => formatter.write_str("lead"),
        }
    }
}

impl FromStr for Authorship {
    type Err = Unvoiced;

    fn from_str(text: &str) -> Result<Authorship, Unvoiced> {
        Self::ALL.into_iter().find(|author| author.to_string() == text).ok_or_else(|| Unvoiced(text.to_owned()))
    }
}

impl Authorship {
    const ALL: [Authorship; 2] = [Authorship::Owner, Authorship::Lead];
}

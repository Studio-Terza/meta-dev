use std::fmt;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callee {
    owner: Option<Name>,
    method: Name,
}

impl Callee {
    pub fn method(method: Name) -> Callee {
        Callee { owner: None, method }
    }

    pub fn associated(owner: Name, method: Name) -> Callee {
        Callee { owner: Some(owner), method }
    }
}

impl fmt::Display for Callee {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.owner {
            Some(owner) => write!(formatter, "{owner}::{}", self.method),
            None => write!(formatter, "{}", self.method),
        }
    }
}

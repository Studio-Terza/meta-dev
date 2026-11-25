use std::fmt;

use crate::end::Identity;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key<'found> {
    takes: Vec<Identity<'found>>,
    gives: Identity<'found>,
}

impl<'found> Key<'found> {
    pub(crate) fn new(takes: Vec<Identity<'found>>, gives: Identity<'found>) -> Key<'found> {
        Key { takes, gives }
    }

    pub(crate) fn fixed(&self) -> Option<&Identity<'found>> {
        self.takes.iter().all(|taken| *taken == self.gives).then_some(&self.gives)
    }
}

impl fmt::Display for Key<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let takes: Vec<String> = self.takes.iter().map(ToString::to_string).collect();
        write!(formatter, "{} -> {}", takes.join(", "), self.gives)
    }
}

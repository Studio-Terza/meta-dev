use std::fmt;

use lint_transition_graph::Answer;

use crate::query::matches::Matches;

#[derive(Debug)]
pub struct Reply<'view, 'found, 'file> {
    matches: Option<Matches<'view, 'file>>,
    answers: Vec<Answer<'view, 'found, 'file>>,
}

impl<'view, 'found, 'file> Reply<'view, 'found, 'file> {
    pub(crate) fn new(matches: Option<Matches<'view, 'file>>, answers: Vec<Answer<'view, 'found, 'file>>) -> Self {
        Reply { matches, answers }
    }
}

impl fmt::Display for Reply<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.matches.iter().try_for_each(|matches| write!(formatter, "{matches}"))?;
        self.answers.iter().try_for_each(|answer| write!(formatter, "{answer}"))
    }
}

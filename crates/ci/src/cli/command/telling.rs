use std::fmt::{self, Debug, Display};
use std::{iter, vec};
use std::process::ExitCode;

use console_ports::{Answer, Shown, Tale};

use super::Failure;
use crate::cli::ports::Exit;

#[derive(Debug)]
pub(crate) enum Told<'step> {
    Said(Shown),
    Ended(Result<Exit, Failure<'step>>),
}

#[derive(Debug)]
pub struct Telling<'step> {
    told: vec::IntoIter<Told<'step>>,
    ended: Option<Answer>,
}

#[derive(Clone, Copy, Debug)]
struct Lined<'said, Said>(&'said Said);

impl<'step> Telling<'step> {
    pub(crate) fn listed(told: Vec<Told<'step>>) -> Telling<'step> {
        Telling { told: told.into_iter(), ended: None }
    }

    pub(crate) fn reported(said: Vec<Shown>, ended: Result<Exit, Failure<'step>>) -> Telling<'step> {
        Telling::listed(said.into_iter().map(Told::Said).chain(iter::once(Told::Ended(ended))).collect())
    }

    pub(crate) fn ended(ended: Result<Exit, Failure<'step>>) -> Telling<'step> {
        Telling::reported(Vec::new(), ended)
    }

    pub(crate) fn refused(refusal: &Failure<'_>) -> Telling<'step> {
        Telling { told: Vec::new().into_iter(), ended: Some(Answer::complained(ExitCode::FAILURE, Lined(refusal))) }
    }

    fn answered(ended: Result<Exit, Failure<'_>>) -> Answer {
        match ended {
            Ok(exit) => Answer::new(ExitCode::from(exit)),
            Err(failure) => Answer::complained(ExitCode::FAILURE, Lined(&failure)),
        }
    }
}

impl Told<'_> {
    pub(crate) fn said(text: impl Display) -> Told<'static> {
        Told::Said(Shown::said(text))
    }
}

impl<'step> Told<'step> {
    pub(crate) fn listed(told: Result<Vec<Told<'step>>, Failure<'step>>) -> Vec<Told<'step>> {
        match told {
            Ok(listed) => listed,
            Err(failure) => vec![Told::Ended(Err(failure))],
        }
    }
}

impl Iterator for Telling<'_> {
    type Item = Shown;

    fn next(&mut self) -> Option<Shown> {
        match self.told.next()? {
            Told::Said(shown) => Some(shown),
            Told::Ended(ended) => self.ended.replace(Telling::answered(ended)).and(None),
        }
    }
}

impl Tale for Telling<'_> {
    fn ended(self) -> Answer {
        self.ended.unwrap_or_else(|| Answer::complained(ExitCode::FAILURE, "the step ended without saying how\n"))
    }
}

impl<Said: Display> Display for Lined<'_, Said> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", self.0)
    }
}

use std::fmt::{self, Debug, Display};
use std::process::ExitCode;
use std::{iter, vec};

use console_ports::{Answer, Shown, Tale};

use super::Failure;
use super::step::mutation::Deferred;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub(crate) enum Told<'step> {
    Said(Shown),
    Ended(Result<Exit, Failure<'step>>),
}

#[derive(Debug)]
pub(crate) struct Telling<'step> {
    told: vec::IntoIter<Told<'step>>,
    deferred: Option<Deferred<'step>>,
    ended: Option<Answer>,
}

#[derive(Debug)]
pub struct Unfolding<'step, P: Ports> {
    telling: Telling<'step>,
    ports: &'step P,
}

#[derive(Clone, Copy, Debug)]
struct Lined<'said, Said>(&'said Said);

impl<'step> Telling<'step> {
    pub(crate) fn listed(told: Vec<Told<'step>>) -> Telling<'step> {
        Telling { told: told.into_iter(), deferred: None, ended: None }
    }

    pub(crate) fn deferring(told: Vec<Told<'step>>, deferred: Deferred<'step>) -> Telling<'step> {
        Telling { deferred: Some(deferred), ..Telling::listed(told) }
    }

    pub(crate) fn reported(said: Vec<Shown>, ended: Result<Exit, Failure<'step>>) -> Telling<'step> {
        Telling::listed(said.into_iter().map(Told::Said).chain(iter::once(Told::Ended(ended))).collect())
    }

    pub(crate) fn ended(ended: Result<Exit, Failure<'step>>) -> Telling<'step> {
        Telling::reported(Vec::new(), ended)
    }

    pub(crate) fn refused(refusal: &Failure<'_>) -> Telling<'step> {
        let ended = Some(Answer::complained(ExitCode::FAILURE, Lined(refusal)));
        Telling { ended, ..Telling::listed(Vec::new()) }
    }

    fn answered(ended: Result<Exit, Failure<'_>>) -> Answer {
        match ended {
            Ok(exit) => Answer::new(ExitCode::from(exit)),
            Err(failure) => Answer::complained(ExitCode::FAILURE, Lined(&failure)),
        }
    }

    fn heard(&mut self, told: Told<'_>) -> Option<Shown> {
        match told {
            Told::Said(shown) => Some(shown),
            Told::Ended(ended) => self.ended.replace(Telling::answered(ended)).and(None),
        }
    }
}

impl<'step, P: Ports> Unfolding<'step, P> {
    pub(crate) fn new(telling: Telling<'step>, ports: &'step P) -> Unfolding<'step, P> {
        Unfolding { telling, ports }
    }

    fn unfolded(&mut self) -> Option<Shown> {
        let deferred = self.telling.deferred.take()?;
        self.telling.told = deferred.unfolded(self.ports).into_iter();
        self.next()
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

impl<P: Ports> Iterator for Unfolding<'_, P> {
    type Item = Shown;

    fn next(&mut self) -> Option<Shown> {
        match self.telling.told.next() {
            Some(told) => self.telling.heard(told),
            None => self.unfolded(),
        }
    }
}

impl<P: Ports> Tale for Unfolding<'_, P> {
    fn ended(self) -> Answer {
        self.telling
            .ended
            .unwrap_or_else(|| Answer::complained(ExitCode::FAILURE, "the step ended without saying how\n"))
    }
}

impl<Said: Display> Display for Lined<'_, Said> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", self.0)
    }
}

use std::marker::PhantomData;

use console_ports::Shown;
use port_core::{ReleaseFailures, Snapshot};

use super::failure::Failure;
use super::request::Asked;
use crate::Outcome;

#[derive(Debug)]
pub enum Said {}

#[derive(Debug)]
pub enum Settled {}

#[derive(Debug)]
pub struct Uttered<State = Said> {
    shown: Shown,
    ended: Result<Outcome, Failure>,
    asked: Vec<Asked>,
    state: PhantomData<State>,
}

impl Uttered<Said> {
    pub(crate) fn new(shown: Shown, ended: Result<Outcome, Failure>) -> Uttered<Said> {
        Uttered { shown, ended, asked: Vec::new(), state: PhantomData }
    }

    pub(crate) fn silent(ended: Result<Outcome, Failure>) -> Uttered<Said> {
        Uttered::new(Shown::default(), ended)
    }

    pub(crate) fn asking(self, asked: Vec<Asked>) -> Uttered<Said> {
        Uttered { asked, ..self }
    }

    pub(crate) fn settled(self, released: Result<Snapshot, ReleaseFailures>) -> Uttered<Settled> {
        let ended = Failure::settled(self.ended, released);
        Uttered { shown: self.shown, ended, asked: self.asked, state: PhantomData }
    }
}

impl<State> Uttered<State> {
    pub fn ended(self) -> Result<Outcome, Failure> {
        self.ended
    }
}

impl<State> AsRef<[Asked]> for Uttered<State> {
    fn as_ref(&self) -> &[Asked] {
        &self.asked
    }
}

impl<State> AsRef<Shown> for Uttered<State> {
    fn as_ref(&self) -> &Shown {
        &self.shown
    }
}

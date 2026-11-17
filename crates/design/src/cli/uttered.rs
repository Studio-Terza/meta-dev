use std::marker::PhantomData;

use console_ports::Shown;
use port_core::{ReleaseFailures, Snapshot};

use super::failure::Failure;
use crate::Outcome;

#[derive(Debug)]
pub enum Said {}

#[derive(Debug)]
pub enum Settled {}

#[derive(Debug)]
pub struct Uttered<State = Said> {
    shown: Shown,
    ended: Result<Outcome, Failure>,
    state: PhantomData<State>,
}

impl Uttered<Said> {
    pub(crate) fn new(shown: Shown, ended: Result<Outcome, Failure>) -> Uttered<Said> {
        Uttered { shown, ended, state: PhantomData }
    }

    pub(crate) fn silent(ended: Result<Outcome, Failure>) -> Uttered<Said> {
        Uttered::new(Shown::default(), ended)
    }

    pub(crate) fn settled(self, released: Result<Snapshot, ReleaseFailures>) -> Uttered<Settled> {
        Uttered { shown: self.shown, ended: Failure::settled(self.ended, released), state: PhantomData }
    }
}

impl<State> Uttered<State> {
    pub fn ended(self) -> Result<Outcome, Failure> {
        self.ended
    }
}

impl<State> AsRef<Shown> for Uttered<State> {
    fn as_ref(&self) -> &Shown {
        &self.shown
    }
}

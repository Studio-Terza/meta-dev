use super::op::{Op, Pass};
use super::run::Run;

mod sealed {
    pub trait Sealed {}
}

pub trait Metered: sealed::Sealed {}

#[derive(Debug)]
pub struct Port<'run, F> {
    filler: F,
    run: &'run Run,
}

impl<F> sealed::Sealed for Port<'_, F> {}
impl<F> Metered for Port<'_, F> {}

impl Run {
    pub fn port<F>(&self, filler: F) -> Port<'_, F> {
        Port { filler, run: self }
    }
}

impl<F> Port<'_, F> {
    pub fn counted_call<O: Op, T>(&self, body: impl FnOnce(&F, Pass<'_, O>) -> T) -> T {
        self.run.count(O::NAME);
        body(&self.filler, Pass::minted())
    }
}

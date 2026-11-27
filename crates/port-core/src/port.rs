use super::held::{Held, Release};
use super::op::{Op, Pass};
use super::run::Run;

mod sealed {
    pub trait Sealed {}
}

pub trait Metered: sealed::Sealed {}

#[derive(Clone, Debug)]
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

impl<'run, F> Port<'run, F> {
    pub fn counted_call<O: Op, T>(&self, body: impl FnOnce(&F, Pass<'_, O>) -> T) -> T {
        self.run.count(O::NAME);
        body(&self.filler, Pass::minted())
    }

    pub fn counted_acquire<O: Op, R: Release, E>(
        &self,
        body: impl FnOnce(&F, Pass<'_, O>) -> Result<R, E>,
    ) -> Result<Held<'run, R>, E> {
        self.run.count(O::NAME);
        body(&self.filler, Pass::minted()).map(|resource| Held::acquired(resource, self.run))
    }
}

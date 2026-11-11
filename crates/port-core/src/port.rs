use super::held::{Held, Release};
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

impl<'run, F> Port<'run, F> {
    pub fn counted_call<O: Op, T>(&self, body: impl FnOnce(&F, Pass<'_, O>) -> T) -> T {
        self.run.count(O::NAME);
        body(&self.filler, Pass::minted())
    }

    pub fn counted_acquire<O: Op, T: Holds>(&self, body: impl FnOnce(&F, Pass<'_, O>) -> T) -> T::Held<'run> {
        self.run.count(O::NAME);
        body(&self.filler, Pass::minted()).held(Acquired(self.run))
    }
}

#[derive(Debug)]
pub struct Acquired<'run>(&'run Run);

pub trait Holds {
    type Held<'run>;

    fn held(self, acquired: Acquired<'_>) -> Self::Held<'_>;
}

impl<R: Release> Holds for R {
    type Held<'run> = Held<'run, R>;

    fn held(self, acquired: Acquired<'_>) -> Self::Held<'_> {
        Held::acquired(self, acquired.0)
    }
}

impl<R: Release> Holds for Option<R> {
    type Held<'run> = Option<Held<'run, R>>;

    fn held(self, acquired: Acquired<'_>) -> Self::Held<'_> {
        self.map(|resource| Held::acquired(resource, acquired.0))
    }
}

impl<R: Release, E> Holds for Result<R, E> {
    type Held<'run> = Result<Held<'run, R>, E>;

    fn held(self, acquired: Acquired<'_>) -> Self::Held<'_> {
        self.map(|resource| Held::acquired(resource, acquired.0))
    }
}

use std::marker::PhantomData;

use port_core::{Rooting, Run};

use crate::{Bounded, Failure, Growth, Scale, Verified, Workload};

#[derive(Debug)]
pub struct Harness<W>(PhantomData<W>);

impl<W: Workload> Harness<W> {
    pub fn verified() -> Result<Verified<W>, Failure> {
        let runs = Scale::ALL.into_iter().map(Self::bounded).collect::<Result<Vec<_>, Failure>>()?;
        let growths = runs
            .split_first()
            .map_or(Ok(Vec::new()), |(base, scaled)| scaled.iter().map(|run| Self::growth(base, run)).collect())?;
        Ok(Verified::new(runs, growths))
    }

    fn bounded(scale: Scale) -> Result<Bounded<W::Done>, Failure> {
        let run = Run::root(Rooting::Sole)?;
        let done = W::run(&run, W::fixture(scale)).map_err(|failure| Failure::Ran(Box::new(failure)))?;
        let snapshot = run.finish().snapshot().clone();
        let within =
            W::bounds(scale).into_iter().map(|bound| bound.checked(&snapshot, scale)).collect::<Result<_, _>>()?;
        Ok(Bounded::new(scale, done, snapshot, within))
    }

    fn growth(base: &Bounded<W::Done>, scaled: &Bounded<W::Done>) -> Result<Growth, Failure> {
        scaled
            .snapshot()
            .iter()
            .map(|counted| {
                W::CLASS.admits(counted.name(), base.snapshot().named(counted.name()), counted.calls(), scaled.scale())
            })
            .collect::<Result<_, _>>()
            .map(|admitted| Growth::new(W::CLASS, scaled.scale(), admitted))
    }
}

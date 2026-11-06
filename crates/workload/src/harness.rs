use std::marker::PhantomData;

use port_core::{Rooting, Run, Snapshot};

use crate::{Failure, Scale, Workload};

#[derive(Debug)]
pub struct Harness<W>(PhantomData<W>);

impl<W: Workload> Harness<W> {
    pub fn verified() -> Result<(), Failure> {
        let measured = Scale::ALL
            .into_iter()
            .map(|scale| Self::measured(scale).map(|snapshot| (scale, snapshot)))
            .collect::<Result<Vec<_>, Failure>>()?;
        measured.iter().skip(1).try_for_each(|(scale, snapshot)| {
            measured.first().map_or(Ok(()), |(_, base)| Self::growth(base, snapshot, *scale))
        })
    }

    fn measured(scale: Scale) -> Result<Snapshot, Failure> {
        let run = Run::root(Rooting::Sole)?;
        W::run(&run, W::fixture(scale)).map_err(|failure| Failure::Ran(Box::new(failure)))?;
        let snapshot = run.finish().snapshot().clone();
        W::bounds(scale).into_iter().try_for_each(|bound| bound.checked(&snapshot, scale))?;
        Ok(snapshot)
    }

    fn growth(base: &Snapshot, scaled: &Snapshot, factor: Scale) -> Result<(), Failure> {
        scaled.iter().try_for_each(|counted| {
            W::CLASS.admits(counted.name(), base.named(counted.name()), counted.calls(), factor)
        })
    }
}

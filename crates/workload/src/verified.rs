use std::any::type_name;
use std::fmt;
use std::marker::PhantomData;

use crate::{Bounded, Growth, Workload};

#[derive(Debug)]
pub struct Verified<W: Workload> {
    runs: Vec<Bounded<W::Done>>,
    growths: Vec<Growth>,
    workload: PhantomData<W>,
}

impl<W: Workload> Verified<W> {
    pub(crate) fn new(runs: Vec<Bounded<W::Done>>, growths: Vec<Growth>) -> Verified<W> {
        Verified { runs, growths, workload: PhantomData }
    }
}

impl<W: Workload> fmt::Display for Verified<W> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{} verified", type_name::<W>())?;
        self.runs.iter().try_for_each(|run| writeln!(formatter, "{run}"))?;
        self.growths.iter().try_for_each(|growth| writeln!(formatter, "{growth}"))
    }
}

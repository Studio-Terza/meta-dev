use std::iter;

use port_core::Held;

use crate::{AgentRun, Clock, Dispatch, DispatchFailure, Due, Finished, Pause};

#[derive(Debug)]
pub struct Watch<'ports, D: Dispatch, C: Clock> {
    dispatch: &'ports D,
    clock: &'ports C,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    Finished(Finished),
    Overdue,
}

impl<'ports, D: Dispatch, C: Clock> Watch<'ports, D, C> {
    pub const PAUSE: Pause = Pause::new(2);

    pub fn over(dispatch: &'ports D, clock: &'ports C) -> Watch<'ports, D, C> {
        Watch { dispatch, clock }
    }

    pub fn watched(&self, mut run: Held<'_, AgentRun>) -> Result<Watched, DispatchFailure> {
        let watched =
            iter::repeat_with(|| self.checked(&mut run)).find_map(Result::transpose).unwrap_or(Ok(Watched::Overdue))?;
        run.end()?;
        Ok(watched)
    }

    fn checked(&self, run: &mut AgentRun) -> Result<Option<Watched>, DispatchFailure> {
        match run.deadline().due(self.clock.now()) {
            Due::Passed => Ok(Some(Watched::Overdue)),
            Due::Pending => Ok(self.dispatch.output(run)?.map(Watched::Finished).or_else(|| self.slept())),
        }
    }

    fn slept(&self) -> Option<Watched> {
        self.clock.waited(Self::PAUSE);
        None
    }
}

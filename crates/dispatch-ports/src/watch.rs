use port_core::Held;

use crate::{
    AgentRun, Clock, Deadline, Dispatch, DispatchFailure, Due, Errored, Fault, Finished, Pause, Unreadable, Usage,
};

#[derive(Debug)]
pub struct Watch<'ports, D: Dispatch, C: Clock> {
    dispatch: &'ports D,
    clock: &'ports C,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    Finished(Finished),
    Stalled(Stall),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Stall {
    #[error("the agent's run ended in error: {0}")]
    Errored(Errored),
    #[error("the agent's output was unreadable: {0}")]
    Unreadable(Unreadable),
    #[error("the run passed its deadline and was killed")]
    Overdue,
    #[error("{0}")]
    Unspawned(Fault),
    #[error("{0}")]
    Failed(Fault),
}

impl Stall {
    #[must_use]
    pub fn usage(&self) -> Option<Usage> {
        match self {
            Stall::Errored(errored) => Some(errored.usage()),
            Stall::Unreadable(_) | Stall::Overdue | Stall::Unspawned(_) | Stall::Failed(_) => None,
        }
    }
}

impl<'ports, D: Dispatch, C: Clock> Watch<'ports, D, C> {
    pub const PAUSE: Pause = Pause::new(2);

    pub fn over(dispatch: &'ports D, clock: &'ports C) -> Watch<'ports, D, C> {
        Watch { dispatch, clock }
    }

    pub fn watched(&self, mut run: Held<'_, AgentRun>) -> Result<Watched, DispatchFailure> {
        let deadline = *AsRef::<Deadline>::as_ref(&*run);
        let watched = deadline
            .checks(self.clock.now(), Self::PAUSE)
            .map(|_planned| self.checked(&mut run))
            .find_map(Result::transpose)
            .unwrap_or(Ok(Watched::Stalled(Stall::Overdue)))?;
        run.end()?;
        Ok(watched)
    }

    fn checked(&self, run: &mut AgentRun) -> Result<Option<Watched>, DispatchFailure> {
        match AsRef::<Deadline>::as_ref(run).due(self.clock.now()) {
            Due::Passed => Ok(Some(Watched::Stalled(Stall::Overdue))),
            Due::Pending => Ok(self.dispatch.output(run)?.map(Watched::from).or_else(|| self.slept())),
        }
    }

    fn slept(&self) -> Option<Watched> {
        self.clock.waited(Self::PAUSE);
        None
    }
}

use std::iter;

use port_core::Held;

use crate::{
    AgentRun, Bell, Clock, Deadline, Dispatch, DispatchFailure, Due, Errored, Fault, Finished, Job, Pause, Prompt,
    Unreadable, Usage,
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

pub trait Watching {
    type Output;

    fn finished(self, finished: &Finished) -> Self::Output;

    fn stalled(self, stall: &Stall) -> Self::Output;
}

impl Watched {
    pub fn visited<W: Watching>(&self, watching: W) -> W::Output {
        match self {
            Watched::Finished(finished) => watching.finished(finished),
            Watched::Stalled(stall) => watching.stalled(stall),
        }
    }
}

pub trait Stalled {
    const ERRORED: Self;
    const UNREADABLE: Self;
    const OVERDUE: Self;
    const UNSPAWNED: Self;
    const FAILED: Self;
}

impl Stall {
    #[must_use]
    pub fn stalled<S: Stalled>(&self) -> S {
        match self {
            Stall::Errored(_) => S::ERRORED,
            Stall::Unreadable(_) => S::UNREADABLE,
            Stall::Overdue => S::OVERDUE,
            Stall::Unspawned(_) => S::UNSPAWNED,
            Stall::Failed(_) => S::FAILED,
        }
    }

    #[must_use]
    pub fn usage(&self) -> Option<Usage> {
        match self {
            Stall::Errored(errored) => Some(errored.usage()),
            Stall::Unreadable(_) | Stall::Overdue | Stall::Unspawned(_) | Stall::Failed(_) => None,
        }
    }
}

impl<'ports, D: Dispatch, C: Clock> Watch<'ports, D, C> {
    pub(crate) const PAUSE: Pause = Pause::new(2);

    pub fn over(dispatch: &'ports D, clock: &'ports C) -> Watch<'ports, D, C> {
        Watch { dispatch, clock }
    }

    pub fn watched(&self, mut run: Held<'_, AgentRun>) -> Result<Watched, DispatchFailure> {
        let deadline = *AsRef::<Deadline>::as_ref(&*run);
        let watched = iter::repeat_with(|| self.clock.now())
            .take_while(|now| deadline.due(*now) == Due::Pending)
            .map(|_now| self.polled(&mut run))
            .find_map(Result::transpose)
            .unwrap_or(Ok(Watched::Stalled(Stall::Overdue)))?;
        run.end()?;
        Ok(watched)
    }

    pub fn fanned(&self, job: &Job, prompts: &[Prompt]) -> Result<Vec<Watched>, DispatchFailure> {
        let running = prompts
            .iter()
            .map(|prompt| self.dispatch.dispatch(job, prompt))
            .collect::<Result<Vec<Held<'_, AgentRun>>, DispatchFailure>>()?;
        running.into_iter().map(|run| self.watched(run)).collect()
    }

    fn polled(&self, run: &mut AgentRun) -> Result<Option<Watched>, DispatchFailure> {
        Ok(self.dispatch.output(run)?.map(Watched::from).or_else(|| self.slept(run.bell())))
    }

    fn slept(&self, bell: &Bell) -> Option<Watched> {
        self.clock.waited(Self::PAUSE, bell);
        None
    }
}

mod answers;
mod boundary;
mod subagent;

use console_ports::Environment;
use dispatch_ports::{Clock, Dispatch, Shell};
use file_ports::Files;
use newtype::Enumerated;
use store_ports::Store;

use super::Ground;
use super::crew::Crew;
use super::ingest::Ingest;
use crate::HookEvent;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::{Observed, Settings, Stopping};
use subagent::Finishing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    Settings,
    On(HookEvent),
}

impl Hook {
    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        self,
        environment: &impl Environment,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let heard = || environment.input().map_err(Failure::Deaf);
        match self {
            Hook::Settings => Settings::listed(HookEvent::VARIANTS).written(),
            Hook::On(HookEvent::Stop) => Hook::stopped(&Stopping::heard(heard()?)?, ground, crew),
            Hook::On(HookEvent::SubagentStop) => Finishing::heard(heard()?)?.answered(&ground, crew),
        }
    }

    fn stopped<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        stopping: &Stopping,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let started = crew.moment();
        let store = ground.store();
        let tasks = stopping.tasks(&ground, crew.now());
        let queue = ground.queue();
        Ingest::read(store, &ground.transcripts())?;
        stopping.decided(crew, &Observed::new(store, tasks, started, queue))
    }
}

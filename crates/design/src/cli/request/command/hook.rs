mod answers;
mod boundary;
mod brief;
mod release;
mod subagent;
mod unroll;

use console_ports::Environment;
use design_messages::EndedTasks;
use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use newtype::Enumerated;
use store_ports::Store;

use super::Ground;
use super::crew::Crew;
use super::ingest::Ingest;
use crate::HookEvent;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::{Observed, Settings, Stopped, Stopping};
use release::Used;
use subagent::Finishing;
use unroll::Running;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    Settings,
    On(HookEvent),
}

impl Hook {
    pub fn answered<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        self,
        environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let heard = || environment.input().map_err(Failure::Deaf);
        match self {
            Hook::Settings => Settings::listed(HookEvent::VARIANTS).written(),
            Hook::On(HookEvent::Stop) => Hook::stopped(&Stopping::heard(heard()?)?, ground, crew),
            Hook::On(HookEvent::SubagentStop) => Finishing::heard(heard()?)?.answered(ground, crew),
            Hook::On(HookEvent::PreToolUse) => Running::heard(heard()?)?.answered(ground, crew),
            Hook::On(HookEvent::PostToolUse) => Used::heard(heard()?)?.answered(ground, crew),
        }
    }

    fn stopped<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        stopping: &Stopping,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let store = ground.store();
        ground.prepared(crew.scratch())?;
        Ingest::read(store, &ground.transcripts())?;
        let stopped = Stopped::from(store.read(EndedTasks)?);
        let tasks = stopping.tasks(ground, crew.now()).map(|tasks| tasks.unstopped(&stopped));
        stopping.decided(crew, &Observed::new(ground, tasks))
    }
}

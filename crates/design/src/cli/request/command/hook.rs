pub(super) mod answers;
mod boundary;

use console_ports::Environment;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use newtype::Enumerated;
use store_ports::Store;

use super::Ground;
use super::ingest::Ingest;
use super::structure::{Crew, Structure};
use crate::cli::failure::Failure;
use crate::{HookEvent, Uttered};
use answers::Answering;
use boundary::{Decision, Prompting, Settings, Stopping};
use design_store::Recall;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    Settings,
    On(HookEvent),
}

impl Hook {
    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock>(
        self,
        environment: &impl Environment,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        match self {
            Hook::Settings => Settings::listed(HookEvent::VARIANTS).written(),
            Hook::On(HookEvent::UserPromptSubmit) => {
                Hook::prompted(&Prompting::heard(environment.input().map_err(Failure::Deaf)?)?, ground)
            }
            Hook::On(HookEvent::Stop) => {
                Hook::stopped(&Stopping::heard(environment.input().map_err(Failure::Deaf)?)?, ground, crew)
            }
        }
    }

    fn prompted<F: Files, S: Store>(prompting: &Prompting, ground: Ground<'_, F, S>) -> Result<Uttered, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts())?;
        let recalled =
            prompting.terms().map(|terms| store.read(Recall::telling(&terms))).transpose()?.unwrap_or_default();
        Decision::told(store, HookEvent::UserPromptSubmit, &recalled, None)
    }

    fn stopped<F: Files, S: Store, D: Dispatch, C: Clock>(
        stopping: &Stopping,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts())?;
        let concluded = Structure::new(store, crew).worked()?.concluded();
        stopping.decided(concluded, store, &Answering::new(store, crew))
    }
}

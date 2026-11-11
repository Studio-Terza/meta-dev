mod boundary;

use std::io::Write;

use console_ports::Environment;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use newtype::Enumerated;
use store_ports::Store;

use super::Ground;
use super::ingest::Ingest;
use super::structure::{Crew, Structure};
use crate::cli::failure::Failure;
use crate::{HookEvent, Outcome};
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
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self {
            Hook::Settings => Settings::listed(HookEvent::VARIANTS).written(out),
            Hook::On(HookEvent::UserPromptSubmit) => {
                Hook::prompted(&Prompting::heard(environment.input().map_err(Failure::Deaf)?)?, ground, out)
            }
            Hook::On(HookEvent::Stop) => {
                Hook::stopped(&Stopping::heard(environment.input().map_err(Failure::Deaf)?)?, ground, crew, out)
            }
        }
    }

    fn prompted<F: Files, S: Store>(
        prompting: &Prompting,
        ground: Ground<'_, F, S>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts())?;
        let recalled =
            prompting.terms().map(|terms| store.read(Recall::telling(&terms))).transpose()?.unwrap_or_default();
        Decision::told(store, HookEvent::UserPromptSubmit, &recalled, out)
    }

    fn stopped<F: Files, S: Store, D: Dispatch, C: Clock>(
        stopping: &Stopping,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts())?;
        let concluded = Structure::new(store, crew).worked()?.concluded();
        stopping.decided(concluded, store, out)
    }
}

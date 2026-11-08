mod boundary;

use std::io::Write;

use console_ports::Environment;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use store_ports::Store;
use strum::VariantArray;

use super::Ground;
use super::ingest::Ingest;
use super::structure::{Crew, Structure};
use crate::cli::failure::Failure;
use crate::{HookEvent, Outcome};
use boundary::{Settings, Stopping};

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
            Hook::On(HookEvent::UserPromptSubmit) => Hook::prompted(ground),
            Hook::On(HookEvent::Stop) => {
                Hook::stopped(&Stopping::heard(environment.input().map_err(Failure::Deaf)?)?, ground, crew, out)
            }
        }
    }

    fn prompted<F: Files, S: Store>(ground: Ground<'_, F, S>) -> Result<Outcome, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts()).map(|_tally| Outcome::Hooked(HookEvent::UserPromptSubmit))
    }

    fn stopped<F: Files, S: Store, D: Dispatch, C: Clock>(
        stopping: &Stopping,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts())?;
        Structure::new(store, crew)
            .worked()?
            .concluded()
            .map_or_else(|halt| stopping.decided(&halt).written(out), |_jobs| Ok(Outcome::Hooked(HookEvent::Stop)))
    }
}

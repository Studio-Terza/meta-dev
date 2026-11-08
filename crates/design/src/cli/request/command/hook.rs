mod boundary;

use std::io::Write;

use design_store::Unstructured;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use store_ports::Store;
use strum::VariantArray;

use super::Ground;
use super::ingest::Ingest;
use super::structure::{Crew, Structure};
use crate::cli::failure::Failure;
use crate::cli::output::Count;
use crate::{HookEvent, Outcome};
use boundary::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    Settings,
    On(HookEvent),
}

impl Hook {
    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock>(
        self,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self {
            Hook::Settings => Settings::listed(HookEvent::VARIANTS).written(out).map(|()| Outcome::Settings),
            Hook::On(HookEvent::UserPromptSubmit) => Hook::prompted(ground),
            Hook::On(HookEvent::Stop) => Hook::stopped(ground, crew, out),
        }
    }

    fn prompted<F: Files, S: Store>(ground: Ground<'_, F, S>) -> Result<Outcome, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts()).map(|_tally| Outcome::Hooked(HookEvent::UserPromptSubmit))
    }

    fn stopped<F: Files, S: Store, D: Dispatch, C: Clock>(
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let store = ground.store();
        Ingest::read(store, &ground.transcripts())?;
        Structure::new(store, crew).worked()?.reported(out)?;
        Hook::covered(store).map(|()| Outcome::Hooked(HookEvent::Stop))
    }

    fn covered(store: &impl Store) -> Result<(), Failure> {
        let lock = store.lock()?;
        let owed = store.read(Unstructured)?;
        lock.end()?;
        owed.first().map_or(Ok(()), |_first| Err(Failure::Unstructured { owed: Count::new(owed.len()) }))
    }
}

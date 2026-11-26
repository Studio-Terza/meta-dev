mod crew;
mod hook;
mod ingest;
mod stops;

use std::path::PathBuf;

use console_ports::{Environment, Shown};
use design_messages::StopTrail;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use store_ports::{Opens, Store};
use strum::VariantArray;

use super::Ground;
use super::persistence::Persistence;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub use crew::Crew;
use hook::Hook;
use ingest::Ingest;
use stops::Trail;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest(Persistence),
    Stops,
    Hook(Hook),
}

impl Command {
    const INGEST: &'static str = "ingest";
    const DRY_RUN: &'static str = "--dry-run";
    const STOPS: &'static str = "stops";
    const HOOK: &'static str = "hook";
    const SETTINGS: &'static str = "--settings";

    pub fn spoken(words: &[&str]) -> Result<Command, Failure> {
        match words {
            [Self::INGEST] => Ok(Command::Ingest(Persistence::File)),
            [Self::INGEST, Self::DRY_RUN] => Ok(Command::Ingest(Persistence::DryRun)),
            [Self::STOPS] => Ok(Command::Stops),
            [Self::HOOK, Self::SETTINGS] => Ok(Command::Hook(Hook::Settings)),
            [Self::HOOK, event] => Hook::named(event).map(Command::Hook),
            _ => Err(Failure::Usage),
        }
    }

    pub fn opened<O: Opens>(self, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Command::Ingest(persistence) => persistence.opened(path, opens),
            Command::Stops | Command::Hook(_) => Persistence::File.opened(path, opens),
        }
    }

    pub fn written<F: Files, S: Store, D: Dispatch, C: Clock>(
        self,
        environment: &impl Environment,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        match self {
            Command::Ingest(_) => Ingest::read(ground.store(), &ground.transcripts())
                .map(|tally| Uttered::new(Shown::said(tally), Ok(Outcome::Ingested))),
            Command::Stops => ground
                .store()
                .read(StopTrail)
                .map(|rows| Uttered::new(Shown::said(Trail::from(rows)), Ok(Outcome::Trailed)))
                .map_err(Failure::from),
            Command::Hook(hook) => hook.answered(environment, ground, crew),
        }
    }
}

impl Hook {
    fn named(event: &str) -> Result<Hook, Failure> {
        HookEvent::VARIANTS
            .iter()
            .find(|known| <&str>::from(**known) == event)
            .map(|known| Hook::On(*known))
            .ok_or(Failure::Usage)
    }
}

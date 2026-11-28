mod crew;
mod hook;
mod ingest;

use std::path::PathBuf;

use console_ports::{Environment, Shown};
use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use store_ports::{Opens, Store};
use strum::VariantArray;

use super::Ground;
use super::persistence::Persistence;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub use crew::Crew;
use hook::Hooked;
use ingest::Ingest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest(Persistence),
    Hook(Hooked),
}

impl Command {
    const INGEST: &'static str = "ingest";
    const DRY_RUN: &'static str = "--dry-run";
    const HOOK: &'static str = "hook";
    const SETTINGS: &'static str = "--settings";

    pub fn spoken(words: &[&str]) -> Result<Command, Failure> {
        match words {
            [Self::INGEST] => Ok(Command::Ingest(Persistence::File)),
            [Self::INGEST, Self::DRY_RUN] => Ok(Command::Ingest(Persistence::DryRun)),
            [Self::HOOK, Self::SETTINGS] => Ok(Command::Hook(Hooked::Settings)),
            [Self::HOOK, event] => Hooked::named(event).map(Command::Hook),
            _ => Err(Failure::Usage),
        }
    }

    pub fn opened<O: Opens>(self, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Command::Ingest(persistence) => persistence.opened(path, opens),
            Command::Hook(_) => Persistence::File.opened(path, opens),
        }
    }

    pub fn written<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        self,
        environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        match self {
            Command::Ingest(_) => Ingest::read(ground.store(), &ground.transcripts())
                .map(|tally| Uttered::new(Shown::said(tally), Ok(Outcome::Ingested))),
            Command::Hook(hook) => hook.answered(environment, ground, crew),
        }
    }
}

impl Hooked {
    fn named(event: &str) -> Result<Hooked, Failure> {
        HookEvent::VARIANTS
            .iter()
            .find(|known| <&str>::from(**known) == event)
            .map(|known| Hooked::On(*known))
            .ok_or(Failure::Usage)
    }
}

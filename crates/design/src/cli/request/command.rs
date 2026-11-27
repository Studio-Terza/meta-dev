mod boundary;
mod crew;
mod hook;
mod ingest;
mod stops;

use std::path::PathBuf;

use console_ports::{Environment, Shown};
use design_messages::StopTrail;
use dispatch_ports::{Clock, Dispatch, Shell};
use file_ports::Files;
use store_ports::{Opens, Store};

use super::Ground;
use super::persistence::Persistence;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
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
    pub fn opened<O: Opens>(self, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Command::Ingest(persistence) => persistence.opened(path, opens),
            Command::Stops | Command::Hook(_) => Persistence::File.opened(path, opens),
        }
    }

    pub fn written<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        self,
        environment: &impl Environment,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
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

mod boundary;
mod crew;
mod hook;
mod ingest;

use std::path::PathBuf;

use console_ports::{Environment, Shown};
use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use store_ports::{Opens, Schema, Store};

use super::Ground;
use super::persistence::Persistence;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
pub(crate) use crew::Crew;
use hook::Hook;
use ingest::Ingest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Ingest(Persistence),
    Hook(Hook),
}

impl Command {
    pub(crate) fn opened<O: Opens>(self, schema: &Schema, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Command::Ingest(persistence) => persistence.opened(schema, path, opens),
            Command::Hook(_) => Persistence::File.opened(schema, path, opens),
        }
    }

    pub(crate) fn written<F: Files + Clone, S: Store, C: Clock, G: Shell>(
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

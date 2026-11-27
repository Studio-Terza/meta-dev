use strum::VariantArray;

use super::Command;
use super::hook::Hook;
use crate::HookEvent;
use crate::cli::failure::Failure;
use crate::cli::request::persistence::Persistence;

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

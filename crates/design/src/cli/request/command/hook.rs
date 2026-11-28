mod answers;
mod boundary;
mod brief;
mod release;
mod reply;
mod subagent;
mod unroll;

use console_ports::Environment;
use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use newtype::Enumerated;
use store_ports::Store;

use super::Ground;
use super::crew::Crew;
use crate::HookEvent;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::{Settings, Stopping};
use release::Used;
use reply::Hook;
use subagent::Finishing;
use unroll::Running;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hooked {
    Settings,
    On(HookEvent),
}

impl Hooked {
    pub fn answered<F: Files + Clone, S: Store, C: Clock, G: Shell>(
        self,
        environment: &impl Environment,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let heard = || environment.input().map_err(Failure::Deaf);
        match self {
            Hooked::Settings => Settings::listed(HookEvent::VARIANTS).written(),
            Hooked::On(HookEvent::Stop) => Stopping::heard(heard()?)?.answered(environment, ground, crew)?.uttered(),
            Hooked::On(HookEvent::SubagentStop) => {
                Finishing::heard(heard()?)?.answered(environment, ground, crew)?.uttered()
            }
            Hooked::On(HookEvent::PreToolUse) => {
                Running::heard(heard()?)?.answered(environment, ground, crew)?.uttered()
            }
            Hooked::On(HookEvent::PostToolUse) => Used::heard(heard()?)?.answered(environment, ground, crew)?.uttered(),
        }
    }
}

mod answers;
mod boundary;
mod brief;
mod release;
mod reply;
mod subagent;
mod unroll;

use console_ports::Heard;
use file_ports::Files;
use newtype::Enumerated;
use store_ports::Store;

use super::Ground;
use super::crew::Tape;
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
    pub fn answered<F: Files + Clone, S: Store>(
        self,
        heard: Option<&Heard>,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Uttered, Failure> {
        let input = || heard.cloned().ok_or(Failure::Usage);
        match self {
            Hooked::Settings => Settings::listed(HookEvent::VARIANTS, &Settings::harness(ground, tape)?).written(),
            Hooked::On(HookEvent::Stop) => Stopping::heard(input()?)?.answered(ground, tape)?.uttered(),
            Hooked::On(HookEvent::SubagentStop) => Finishing::heard(input()?)?.answered(ground, tape)?.uttered(),
            Hooked::On(HookEvent::PreToolUse) => Running::heard(input()?)?.answered(ground, tape)?.uttered(),
            Hooked::On(HookEvent::PostToolUse) => Used::heard(input()?)?.answered(ground, tape)?.uttered(),
        }
    }
}

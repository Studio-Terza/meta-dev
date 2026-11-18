use dispatch_ports::Pid;
use invariants::{Ignoring, ignored};

ignored! {
    enum Ignored {
        BeyondRange => "a dispatch count past the u32 range saturates at its maximum pid",
    }
}

#[derive(Debug)]
pub(super) struct Spawns;

impl Spawns {
    const FIRST: u32 = 100;

    pub(super) fn pid(dispatched: usize) -> Pid {
        Pid::new(
            u32::try_from(dispatched)
                .ignoring(Ignored::BeyondRange)
                .map_or(u32::MAX, |count| count.saturating_add(Self::FIRST)),
        )
    }
}

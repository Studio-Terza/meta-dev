use dispatch_ports::Pid;

#[derive(Debug)]
pub(super) struct Spawns;

impl Spawns {
    const FIRST: u32 = 100;

    pub(super) fn pid(dispatched: usize) -> Pid {
        Pid::new(u32::try_from(dispatched).map_or(u32::MAX, |count| count.saturating_add(Self::FIRST)))
    }
}

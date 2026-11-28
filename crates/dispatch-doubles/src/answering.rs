mod boundary;

use std::fmt;
use std::sync::Mutex;

use dispatch_ports::{Call, DispatchFailure, Ran, ShellFiller, Wall, shell_ops};
use invariants::Unpoisoned;
use port_core::Pass;

pub use boundary::Scripted;
use boundary::{Asked, Log};

pub struct Answering {
    scripted: Vec<Scripted>,
    asked: Mutex<Vec<Asked>>,
    start: Wall,
}

impl Answering {
    #[must_use]
    pub fn new(scripted: Vec<Scripted>, start: Wall) -> Answering {
        Answering { scripted, asked: Mutex::default(), start }
    }

    fn answered(&self, call: &Call) -> Result<Ran, DispatchFailure> {
        let asked = Asked::of(call, self.start);
        let mut log = self.asked.lock().unpoisoned();
        let earlier = asked.repeats(log.iter());
        log.push(asked);
        let matching: Vec<&Scripted> =
            self.scripted.iter().filter(|scripted| scripted.answers(call).is_some()).collect();
        matching.get(earlier).or(matching.last()).map(|scripted| scripted.ran()).ok_or(DispatchFailure::Unjoined)
    }
}

impl ShellFiller for &Answering {
    fn ran(&self, _pass: Pass<'_, shell_ops::Ran>, call: &Call) -> Result<Ran, DispatchFailure> {
        self.answered(call)
    }

    fn fanned(&self, _pass: Pass<'_, shell_ops::Fanned>, calls: &[Call]) -> Vec<Result<Ran, DispatchFailure>> {
        calls.iter().map(|call| self.answered(call)).collect()
    }
}

impl fmt::Debug for Answering {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = self.asked.lock().unpoisoned();
        fmt::Debug::fmt(&Log::from(log.as_slice()), formatter)
    }
}

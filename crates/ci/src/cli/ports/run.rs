use std::time::Duration;

use super::exit::Exit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    exit: Exit,
    elapsed: Duration,
}

impl Run {
    #[must_use]
    pub fn new(exit: Exit, elapsed: Duration) -> Run {
        Run { exit, elapsed }
    }

    #[must_use]
    pub fn exit(&self) -> Exit {
        self.exit
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

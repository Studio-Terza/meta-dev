use std::time::Duration;

use super::exit::Exit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    exit: Exit,
    elapsed: Duration,
}

impl Run {
    #[must_use]
    pub const fn new(exit: Exit, elapsed: Duration) -> Self {
        Self { exit, elapsed }
    }

    #[must_use]
    pub const fn exit(&self) -> Exit {
        self.exit
    }

    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

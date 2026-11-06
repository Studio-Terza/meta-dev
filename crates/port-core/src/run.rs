use std::sync::Arc;

use crate::{Metered, Snapshot, Tally};

#[derive(Debug, Default)]
pub struct Run {
    tally: Arc<Tally>,
}

impl Run {
    #[must_use]
    pub fn meter<P>(&self, port: P) -> Metered<P> {
        Metered::sharing(port, Arc::clone(&self.tally))
    }

    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        self.tally.snapshot()
    }
}

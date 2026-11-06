use std::sync::Arc;

use crate::{Entry, Op, Tally};

#[derive(Debug)]
pub struct Metered<P> {
    port: P,
    tally: Arc<Tally>,
}

impl<P> Metered<P> {
    pub(crate) fn sharing(port: P, tally: Arc<Tally>) -> Metered<P> {
        Metered { port, tally }
    }

    #[must_use]
    pub fn tally(&self) -> &Tally {
        &self.tally
    }

    pub fn call<O: Op<Port = P> + 'static, R>(&self, body: impl FnOnce(&P, Entry<O>) -> R) -> R {
        self.tally.record::<O>();
        body(&self.port, Entry::admitted())
    }
}

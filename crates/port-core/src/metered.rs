use crate::{Entry, Op, Tally};

#[derive(Debug)]
pub struct Metered<P> {
    port: P,
    tally: Tally,
}

impl<P> Metered<P> {
    pub fn new(port: P) -> Metered<P> {
        Metered { port, tally: Tally::default() }
    }

    pub fn call<O: Op<Port = P>, R>(&self, body: impl FnOnce(&P, Entry<O>) -> R) -> R {
        self.tally.record(O::NAME);
        body(&self.port, Entry::admitted())
    }

    #[must_use]
    pub fn tally(&self) -> &Tally {
        &self.tally
    }
}

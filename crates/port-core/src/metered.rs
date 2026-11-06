use crate::{Entry, Op, Tally};

#[derive(Debug)]
pub struct Metered<P> {
    port: P,
}

impl<P: 'static> Metered<P> {
    pub fn new(port: P) -> Metered<P> {
        Metered { port }
    }

    pub fn call<O: Op<Port = P>, R>(&self, body: impl FnOnce(&P, Entry<O>) -> R) -> R {
        Tally::<P>::record(O::NAME);
        body(&self.port, Entry::admitted())
    }
}

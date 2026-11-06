use port_core::{Calls, Op, OpName, Snapshot};

use crate::{Failure, Scale};

#[derive(Clone, Copy, Debug)]
pub struct Bound {
    name: OpName,
    limit: Calls,
}

impl Bound {
    #[must_use]
    pub fn at_most<O: Op>(limit: Calls) -> Bound {
        Bound { name: O::NAME, limit }
    }

    pub fn checked(self, snapshot: &Snapshot, scale: Scale) -> Result<(), Failure> {
        let calls = snapshot.named(self.name);
        Some(calls)
            .filter(|made| *made > self.limit)
            .map_or(Ok(()), |made| Err(Failure::Exceeded { scale, name: self.name, limit: self.limit, calls: made }))
    }
}

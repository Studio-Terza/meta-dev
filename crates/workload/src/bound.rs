use port_core::{Calls, Op, OpName, Snapshot};

use crate::{Failure, Scale, Within};

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

    pub fn checked(self, snapshot: &Snapshot, scale: Scale) -> Result<Within, Failure> {
        let calls = snapshot.named(self.name);
        Some(calls)
            .filter(|made| *made <= self.limit)
            .map(|made| Within::new(self.name, made, self.limit))
            .ok_or(Failure::Exceeded { scale, name: self.name, limit: self.limit, calls })
    }
}

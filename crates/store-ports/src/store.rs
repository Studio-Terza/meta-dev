use port_core::{Held, port};

use crate::store_lock::StoreLock;
use crate::{Batch, Busy, Created, Query, Record, StoreFailure, Written};

#[port]
pub trait Store {
    #[op(Q)]
    fn read<Q: Query>(&self, query: Q) -> Result<Q::Rows, StoreFailure>;

    #[op(Q)]
    fn write<Q: Batch>(&self, batch: Q) -> Result<Written, Q::Failure>;

    #[op(Created<R>)]
    fn create<R: Record>(&self, created: Created<R>) -> Result<R, StoreFailure>;

    fn lock(&self) -> Result<Held<'_, StoreLock>, Busy>;
}

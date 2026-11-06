use port_core::{Held, port};

use crate::store_lock::StoreLock;
use crate::{Batch, Busy, Query, StoreFailure, Written};

#[port]
pub trait Store {
    #[op(Q)]
    fn read<Q: Query>(&self, query: Q) -> Result<Q::Rows, StoreFailure>;

    #[op(Q)]
    fn write<Q: Batch>(&self, batch: Q) -> Result<Written, StoreFailure>;

    fn lock(&self) -> Result<Held<'_, StoreLock>, Busy>;
}

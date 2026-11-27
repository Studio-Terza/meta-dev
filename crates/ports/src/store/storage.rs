use crate::store::store_lock::StoreLock;
use crate::store::{Batch, Busy, Created, Query, Record, StoreFailure};
use port_core::{Held, port};

#[port]
pub trait Store {
    #[op(Q)]
    fn read<Q: Query>(&self, query: Q) -> Result<Q::Rows, StoreFailure>;

    #[op(Q)]
    fn write<Q: Batch>(&self, batch: Q) -> Result<Q::Done, Q::Failure>;

    #[op(Created<R>)]
    fn create<R: Record>(&self, created: Created<R>) -> Result<R, StoreFailure>;

    fn lock(&self) -> Result<Held<'_, StoreLock>, Busy>;
}

use crate::{Batch, Migrator, StoreFailure, Version};

pub trait Schema: 'static {
    type Rewrite: Batch<Failure = StoreFailure, Done = ()>;

    fn migrator() -> &'static Migrator;

    fn rewrite(pending: Version) -> Option<Self::Rewrite>;
}

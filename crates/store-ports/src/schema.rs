use crate::{Batch, Migrator, StoreFailure, Version};

pub trait Schema: 'static {
    type Rewrite: Batch<Failure = StoreFailure>;

    fn migrator() -> &'static Migrator;

    fn rewrite(before: Version) -> Option<Self::Rewrite>;
}

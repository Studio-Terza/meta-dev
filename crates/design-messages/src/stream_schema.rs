mod anchored;
mod boundary;
mod resegmented;

use std::sync::LazyLock;

use store_ports::{Migrator, Schema, Version};

use anchored::Anchored;
pub use boundary::Rewrite;
use resegmented::Resegmented;

static MIGRATOR: LazyLock<Migrator> = LazyLock::new(|| store_ports::migrate!());

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    type Rewrite = Rewrite;

    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }

    fn rewrite(pending: Version) -> Option<Rewrite> {
        (pending == Resegmented::PENDING)
            .then_some(Rewrite::Resegmented(Resegmented))
            .or_else(|| (pending == Anchored::PENDING).then_some(Rewrite::Anchored(Anchored)))
    }
}

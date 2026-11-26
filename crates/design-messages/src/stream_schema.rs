mod boundary;

use std::sync::LazyLock;

use store_ports::{Migrator, Schema, Version};

pub use boundary::Rewrite;

static MIGRATOR: LazyLock<Migrator> = LazyLock::new(|| store_ports::migrate!());

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    type Rewrite = Rewrite;

    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }

    fn rewrite(_: Version) -> Option<Rewrite> {
        None
    }
}

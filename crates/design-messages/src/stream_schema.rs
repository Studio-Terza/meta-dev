use std::sync::LazyLock;

use ports::store::{Migrator, Schema};

static MIGRATOR: LazyLock<Migrator> = LazyLock::new(|| ports::store::migrate!());

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }
}

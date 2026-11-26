use std::sync::LazyLock;

use store_ports::{Migrator, Schema};

static MIGRATOR: LazyLock<Migrator> = LazyLock::new(|| store_ports::migrate!());

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }
}

mod resegmented;

use store_ports::{Migrator, Schema, Version};

use resegmented::Resegmented;

static MIGRATOR: Migrator = store_ports::migrate!();

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    type Rewrite = Resegmented;

    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }

    fn rewrite(pending: Version) -> Option<Resegmented> {
        (pending == Resegmented::PENDING).then_some(Resegmented)
    }
}

use sqlx::migrate::Migrator;
use store_ports::Schema;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

impl Schema for StreamSchema {
    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }
}

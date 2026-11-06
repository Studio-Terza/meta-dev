use crate::Migrator;

pub trait Schema {
    fn migrator() -> &'static Migrator;
}

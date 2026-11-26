use crate::Migrator;

pub trait Schema: 'static {
    fn migrator() -> &'static Migrator;
}

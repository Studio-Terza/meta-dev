use crate::store::Migrator;

pub trait Schema: 'static {
    fn migrator() -> &'static Migrator;
}

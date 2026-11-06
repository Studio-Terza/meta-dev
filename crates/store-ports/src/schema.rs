use sqlx::migrate::Migrator;

pub trait Schema {
    fn migrator() -> &'static Migrator;
}

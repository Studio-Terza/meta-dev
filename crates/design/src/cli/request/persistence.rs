use std::path::PathBuf;

use design_store::StreamSchema;
use store_ports::Sqlite;

use super::projects::Located;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Persistence {
    #[default]
    File,
    DryRun,
}

impl Persistence {
    pub fn store(self, located: &Located, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        match self {
            Persistence::File => located.store(opened),
            Persistence::DryRun => Sqlite::in_memory::<StreamSchema>(),
        }
    }
}

use std::path::PathBuf;

use design_store::StreamSchema;
use store_ports::Sqlite;

use super::Token;
use crate::cli::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Persistence {
    File,
    DryRun,
}

impl TryFrom<&[Token]> for Persistence {
    type Error = Failure;

    fn try_from(rest: &[Token]) -> Result<Persistence, Failure> {
        rest.iter().try_fold(Persistence::File, |earlier, token| token.persisted(earlier))
    }
}

impl Persistence {
    pub fn then(self, next: Persistence) -> Result<Persistence, Failure> {
        match self {
            Persistence::File => Ok(next),
            Persistence::DryRun => Err(Failure::Usage),
        }
    }

    pub fn opened(self, path: PathBuf, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        match self {
            Persistence::File => opened(path),
            Persistence::DryRun => Sqlite::in_memory::<StreamSchema>(),
        }
    }
}

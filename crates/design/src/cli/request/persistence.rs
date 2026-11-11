use std::path::PathBuf;

use design_store::StreamSchema;
use store_ports::Opens;

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

    pub fn opened<O: Opens>(self, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Persistence::File => opens.file::<StreamSchema>(path),
            Persistence::DryRun => opens.in_memory::<StreamSchema>(),
        }
    }
}

use std::path::PathBuf;

use design_messages::StreamSchema;
use ports::store::Opens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Persistence {
    File,
    DryRun,
}

impl Persistence {
    pub fn opened<O: Opens>(self, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Persistence::File => opens.file::<StreamSchema>(path),
            Persistence::DryRun => opens.in_memory::<StreamSchema>(),
        }
    }
}

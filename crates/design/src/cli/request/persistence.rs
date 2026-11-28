use std::path::PathBuf;

use store_ports::{Opens, Schema};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Persistence {
    File,
    DryRun,
}

impl Persistence {
    pub(crate) fn opened<O: Opens>(self, schema: &Schema, path: PathBuf, opens: &O) -> O::Store {
        match self {
            Persistence::File => opens.file(schema, path),
            Persistence::DryRun => opens.in_memory(schema),
        }
    }
}

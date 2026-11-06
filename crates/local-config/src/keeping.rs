use std::path::Path;

use file_ports::Files;

use crate::{ConfigFailure, Kept, LocalConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keeping {
    Write,
    Plan,
}

impl Keeping {
    pub(crate) fn settled(self, config: &LocalConfig, files: &impl Files, meta: &Path) -> Result<Kept, ConfigFailure> {
        match self {
            Keeping::Write => config.written(files, meta).map(|_written| Kept::Written),
            Keeping::Plan => Ok(Kept::Planned),
        }
    }
}

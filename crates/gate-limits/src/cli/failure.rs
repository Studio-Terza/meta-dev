use std::ffi::OsStr;
use std::io;
use std::path::Path;

use crate::cli::document::Document;
use crate::cli::gate::Seconds;
use crate::cli::stage::Stage;

#[derive(Debug, thiserror::Error)]
pub enum Failure<'cli> {
    #[error("cannot read {path}: {source}", path = path.display())]
    Io { path: &'cli Path, source: io::Error },
    #[error("cannot parse {file}: {source}")]
    Parse { file: Document, source: toml::de::Error },
    #[error("stage {stage} has no usable limit: it needs numeric warn and block with warn not above block")]
    Unlimited { stage: Stage },
    #[error("stage {stage} has a limit but was not measured")]
    Unmeasured { stage: Stage },
    #[error("stage {name} was asked for but has no limit", name = name.display())]
    UnknownStage { name: &'cli OsStr },
    #[error("no stage has a limit")]
    Empty,
    #[error("stage {stage} has warn and block levels, which only the owner changes; only a record is lowered")]
    Levels { stage: Stage },
    #[error(
        "stage {stage} took {measured}, above its record {record}; a record is never raised, so nothing was written"
    )]
    Above { stage: Stage, measured: Seconds, record: Seconds },
}

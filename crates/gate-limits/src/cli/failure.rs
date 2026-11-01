use std::io;
use std::path::PathBuf;

use crate::cli::document::Document;
use crate::cli::stage::Stage;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot parse {file}: {source}")]
    Parse { file: Document, source: toml::de::Error },
    #[error("stage {stage} has no usable limit: it needs numeric warn and block with warn not above block")]
    Unlimited { stage: Stage },
    #[error("stage {stage} has a limit but was not measured")]
    Unmeasured { stage: Stage },
    #[error("no stage has a limit")]
    Empty,
}

use std::path::PathBuf;

use design_model::Project;
use file_ports::FileFailure;

#[derive(Debug, thiserror::Error)]
pub enum ConfigFailure {
    #[error("cannot reach .meta/local.toml: {0}")]
    Files(#[from] FileFailure),
    #[error(".meta/local.toml is not a local config: {0}")]
    Unparsed(#[from] toml::de::Error),
    #[error("cannot write the local config: {0}")]
    Unwritten(#[from] toml::ser::Error),
    #[error(
        "this repository is already initialised as project {project} with workspace {workspace:?}; remove .meta/local.toml to change it"
    )]
    Elsewhere { project: Project, workspace: PathBuf },
}

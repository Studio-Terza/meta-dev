use std::path::PathBuf;

use file_ports::FileFailure;

use crate::terms::Line;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Read { path: PathBuf, source: FileFailure },
    #[error("cannot parse {path}:{line}: {source}", path = path.display())]
    Parse { path: PathBuf, line: Line, source: syn::Error },
    #[error("cannot parse {path}: {source}", path = path.display())]
    Manifest { path: PathBuf, source: toml::de::Error },
    #[error("no Rust sources found under {path}", path = path.display())]
    NoSources { path: PathBuf },
    #[error("cannot start the lint threads: {0}")]
    Threads(rayon::ThreadPoolBuildError),
}

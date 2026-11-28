use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Unread { path: PathBuf, source: io::Error },
    #[error("cannot parse {path}:{line}: {source}", path = path.display())]
    Parse { path: PathBuf, line: usize, source: syn::Error },
    #[error("cannot parse {path}: {source}", path = path.display())]
    Manifest { path: PathBuf, source: toml_ports::TomlFailure },
    #[error("no Rust sources found under {path}", path = path.display())]
    NoSources { path: PathBuf },
    #[error("cannot start the lint threads: {0}")]
    Threads(rayon::ThreadPoolBuildError),
}

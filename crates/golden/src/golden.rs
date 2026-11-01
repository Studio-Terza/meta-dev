use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::{Difference, Failure, Mode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Golden(PathBuf);

impl Golden {
    const DIRECTORY: &'static str = "tests/goldens";
    const PACKAGE_VARIABLE: &'static str = "CARGO_MANIFEST_DIR";

    #[must_use]
    pub fn located_at(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn in_package(package: impl AsRef<Path>, name: impl AsRef<Path>) -> Result<Self, Failure> {
        let name = name.as_ref();
        name.components()
            .all(|part| matches!(part, Component::Normal(_)))
            .then(|| Self(package.as_ref().join(Self::DIRECTORY).join(name)))
            .ok_or_else(|| Failure::Escapes { name: name.to_path_buf() })
    }

    pub fn in_running_package(name: impl AsRef<Path>) -> Result<Self, Failure> {
        let package = env::var_os(Self::PACKAGE_VARIABLE).ok_or(Failure::NoPackage)?;
        Self::in_package(package, name)
    }

    pub fn verify(&self, actual: &str) -> Result<(), Failure> {
        self.verify_in(Mode::current()?, actual)
    }

    pub fn verify_in(&self, mode: Mode, actual: &str) -> Result<(), Failure> {
        match mode {
            Mode::Check => {
                let recorded = fs::read_to_string(&self.0).map_err(|source| self.unread(source))?;
                Difference::between(&recorded, actual).map_or(Ok(()), |difference| {
                    Err(Failure::Differs { path: self.0.clone(), difference: Box::new(difference) })
                })
            }
            Mode::Record => self.written(actual).map_err(|source| Failure::Unwritable { path: self.0.clone(), source }),
        }
    }

    fn written(&self, actual: &str) -> io::Result<()> {
        fs::create_dir_all(self.0.parent().unwrap_or(Path::new(""))).and_then(|()| fs::write(&self.0, actual))
    }

    fn unread(&self, source: io::Error) -> Failure {
        match source.kind() {
            io::ErrorKind::NotFound => Failure::Missing { path: self.0.clone() },
            _ => Failure::Unreadable { path: self.0.clone(), source },
        }
    }
}

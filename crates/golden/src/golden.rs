use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use newtype::Newtype;

use crate::{Difference, Failure, Mode, Output};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Golden {
    package: PathBuf,
    path: PathBuf,
}

impl Golden {
    const DIRECTORY: &'static str = "tests/goldens";
    const PACKAGE_VARIABLE: &'static str = "CARGO_MANIFEST_DIR";
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn in_package(package: impl AsRef<Path>, name: impl AsRef<Path>) -> Result<Self, Failure> {
        let name = name.as_ref();
        name.components()
            .all(|part| matches!(part, Component::Normal(_)))
            .then(|| Self {
                package: package.as_ref().to_path_buf(),
                path: package.as_ref().join(Self::DIRECTORY).join(name),
            })
            .ok_or_else(|| Failure::Escapes { name: name.to_path_buf() })
    }

    pub fn in_running_package(name: impl AsRef<Path>) -> Result<Self, Failure> {
        let package = env::var_os(Self::PACKAGE_VARIABLE).ok_or(Failure::NoPackage)?;
        Self::in_package(package, name)
    }

    pub fn verify(&self, actual: Output<'_>) -> Result<(), Failure> {
        self.verify_in(Mode::current()?, actual)
    }

    pub fn verify_in(&self, mode: Mode, actual: Output<'_>) -> Result<(), Failure> {
        match mode {
            Mode::Check => {
                let recorded = fs::read_to_string(&self.path).map_err(|source| self.unread(source))?;
                Difference::between(Output::new(&recorded), actual).map_or(Ok(()), |difference| {
                    Err(Failure::Differs { path: self.path.clone(), difference: Box::new(difference) })
                })
            }
            Mode::Record => self
                .chosen_for_package()
                .ok_or_else(|| Failure::NotChosen { path: self.path.clone() })?
                .written(actual)
                .map_err(|source| Failure::Unwritable { path: self.path.clone(), source }),
        }
    }

    fn chosen_for_package(&self) -> Option<&Self> {
        (!self.package.join(Self::MANIFEST).exists() || Mode::current().ok() == Some(Mode::Record)).then_some(self)
    }

    fn written(&self, actual: Output<'_>) -> io::Result<()> {
        fs::create_dir_all(self.path.parent().unwrap_or(Path::new("")))
            .and_then(|()| fs::write(&self.path, actual.into_inner()))
    }

    fn unread(&self, source: io::Error) -> Failure {
        match source.kind() {
            io::ErrorKind::NotFound => Failure::Missing { path: self.path.clone() },
            _ => Failure::Unreadable { path: self.path.clone(), source },
        }
    }
}

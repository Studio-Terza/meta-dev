use std::io;
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;
use file_ports::{Bytes, FileFailure, Files, Kind};
use newtype::Newtype;

use crate::{Difference, Failure, Mode, Output};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Golden {
    package: PathBuf,
    path: PathBuf,
}

impl Golden {
    const DIRECTORY: &'static str = "tests/goldens";
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

    pub fn verify(self, environment: &impl Environment, files: &impl Files, actual: Output<'_>) -> Result<(), Failure> {
        let mode = Mode::current(environment)?;
        self.verify_in(environment, files, mode, actual)
    }

    pub fn verify_in(
        self,
        environment: &impl Environment,
        files: &impl Files,
        mode: Mode,
        actual: Output<'_>,
    ) -> Result<(), Failure> {
        match mode {
            Mode::Check => self.checked(files, actual),
            Mode::Record => {
                let written = (files.kind(&self.package.join(Self::MANIFEST)) == Kind::Missing
                    || Mode::current(environment).ok() == Some(Mode::Record))
                .then(|| self.written(files, actual));
                self.recorded(written)
            }
        }
    }

    fn recorded(self, written: Option<Result<(), FileFailure>>) -> Result<(), Failure> {
        match written {
            Some(written) => {
                written.map_err(|failure| Failure::Unwritable { path: self.path, source: io::Error::from(failure) })
            }
            None => Err(Failure::NotChosen { path: self.path }),
        }
    }

    fn checked(self, files: &impl Files, actual: Output<'_>) -> Result<(), Failure> {
        match files.read_text(&self.path) {
            Ok(recorded) => Difference::between(Output::new(recorded.as_ref()), actual).map_or(Ok(()), |difference| {
                Err(Failure::Differs { path: self.path, difference: Box::new(difference) })
            }),
            Err(failure) => Err(self.unread(files, failure)),
        }
    }

    fn written(&self, files: &impl Files, actual: Output<'_>) -> Result<(), FileFailure> {
        files
            .create_directories(self.path.parent().unwrap_or(Path::new("")))
            .and_then(|()| files.write(&self.path, Bytes::from(actual.into_inner())))
    }

    fn unread(self, files: &impl Files, failure: FileFailure) -> Failure {
        if files.kind(&self.path) == Kind::Missing {
            Failure::Missing { path: self.path }
        } else {
            Failure::Unreadable { path: self.path, source: io::Error::from(failure) }
        }
    }
}

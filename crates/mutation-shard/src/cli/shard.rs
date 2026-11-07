mod failure;
mod matrix;
mod mutant_result;
mod outcomes;
mod title;
mod verdict;

use std::ffi::OsString;
use std::path::PathBuf;

use file_ports::Files;

use matrix::Matrix;
use outcomes::Outcomes;

pub use failure::Failure;

#[derive(Debug)]
pub struct Shard {
    directory: PathBuf,
    destination: PathBuf,
}

impl Shard {
    const ARGUMENTS: usize = 2;

    pub fn from_arguments(arguments: &[OsString]) -> Result<Self, Failure> {
        <&[OsString; Self::ARGUMENTS]>::try_from(arguments)
            .map(|[directory, destination]| Self {
                directory: PathBuf::from(directory),
                destination: PathBuf::from(destination),
            })
            .map_err(Failure::Usage)
    }

    pub fn write_matrix(&self, files: &impl Files) -> Result<(), Failure> {
        let outcomes = Outcomes::read(files, &self.directory)?;
        let logs = outcomes.logs(files, &self.directory);
        Matrix::new(outcomes.baseline(&logs), outcomes.mutants(&logs)).write(files, &self.destination)
    }
}

mod failure;
mod matrix;
mod mutant_result;
mod outcomes;
mod verdict;

use std::ffi::OsString;
use std::path::PathBuf;

use file_ports::Files;

use matrix::Matrix;
use outcomes::Outcomes;

use failure::Failure;

#[derive(Debug)]
pub struct Shard {
    directory: PathBuf,
    destination: PathBuf,
}

impl Shard {
    pub fn from_arguments(arguments: &[OsString]) -> Result<Shard, Failure> {
        match arguments {
            [directory, destination] => {
                Ok(Shard { directory: PathBuf::from(directory), destination: PathBuf::from(destination) })
            }
            _ => Err(Failure::Usage),
        }
    }

    pub fn write_matrix(&self, files: &impl Files) -> Result<(), Failure> {
        let outcomes = Outcomes::read(files, &self.directory)?;
        Matrix::new(outcomes.baseline(files, &self.directory), outcomes.mutants(files, &self.directory))
            .write(files, &self.destination)
    }
}

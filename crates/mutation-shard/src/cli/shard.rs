mod failure;
mod matrix;
mod mutant_result;
mod outcomes;

use std::ffi::OsString;
use std::path::PathBuf;

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

    pub fn write_matrix(&self) -> Result<(), Failure> {
        let outcomes = Outcomes::read(&self.directory)?;
        Matrix::new(outcomes.baseline(&self.directory), outcomes.mutants(&self.directory)).write(&self.destination)
    }
}

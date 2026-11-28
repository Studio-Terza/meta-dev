mod boundary;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::Exit;
use crate::failure::Failure;
use crate::ports::Disk;
use boundary::{Matrix, Ran, Report, Shard};

#[derive(Debug)]
pub struct Kills {
    report: PathBuf,
    shards: Vec<PathBuf>,
}

impl Kills {
    pub(crate) fn new(report: &OsString, shards: &[OsString]) -> Kills {
        Kills { report: PathBuf::from(report), shards: shards.iter().map(PathBuf::from).collect() }
    }

    pub(crate) fn run(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let shards = self.shards.iter().map(|shard| Shard::read(shard)).collect::<Result<Vec<Vec<Ran>>, Failure>>()?;
        let report = Report::from(shards.iter().flatten().fold(Matrix::default(), Matrix::with));
        Disk::write(&self.report, &report.json()?)?;
        writeln!(out, "{report}")?;
        Ok(Exit::Passed)
    }
}

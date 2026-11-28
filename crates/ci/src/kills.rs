mod log;
mod outcomes;
mod report;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use console_ports::Answer;
use pipeline::Pipeline;

use crate::failure::Failure;
use crate::kills::outcomes::{Ran, Shard};
use crate::kills::report::{Matrix, Report};
use crate::ports::Disk;

#[derive(Debug)]
pub struct Kills {
    report: PathBuf,
    shards: Vec<PathBuf>,
}

#[derive(Debug)]
struct KillMatrix;

#[derive(Debug)]
struct Reported {
    json: String,
    summary: String,
}

impl Kills {
    pub(crate) fn new(report: &OsString, shards: &[OsString]) -> Kills {
        Kills { report: PathBuf::from(report), shards: shards.iter().map(PathBuf::from).collect() }
    }

    pub(crate) fn answered(&self) -> Result<Answer, Failure> {
        let shards = self.shards.iter().map(|shard| Shard::read(shard)).collect::<Result<Vec<Vec<Ran>>, Failure>>()?;
        let reported = KillMatrix.run(shards.into_iter().flatten())?;
        Disk::write(&self.report, &reported.json)?;
        Ok(Answer::new(ExitCode::SUCCESS).showing(reported.summary))
    }
}

impl Pipeline for KillMatrix {
    type Unit = Ran;
    type Parsed = Matrix;
    type Measured = Report;
    type Act = Result<Reported, Failure>;

    fn parsed(&self, ran: Ran) -> Matrix {
        Matrix::of(&ran)
    }

    fn measured(&self, matrix: Matrix) -> Report {
        Report::from(matrix)
    }

    fn act(&self, report: Report) -> Result<Reported, Failure> {
        Ok(Reported { json: report.json()?, summary: format!("{report}\n") })
    }
}

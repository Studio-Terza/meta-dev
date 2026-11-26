use crate::cli::command::Telling;
use console_ports::Shown;
mod job;
mod repository;
mod run_id;

use std::ffi::OsString;
use std::fmt;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Utf8};
use crate::cli::ports::{Exit, Invocation, Ports};
use grammar_output::Record;
use job::Job;
use repository::Repository;
use run_id::RunId;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct WallTimes {
    #[field(Utf8, via = Repository::new)]
    repository: Repository,
    #[field(Utf8, via = RunId::new)]
    run: RunId,
    #[field(Utf8, via = Job::new)]
    job: Job,
}

impl WallTimes {
    fn invocation(&self) -> Invocation {
        Invocation::new(
            "gh",
            [
                "api".to_owned(),
                format!("repos/{}/actions/runs/{}/jobs", self.repository, self.run),
                "--jq".to_owned(),
                format!(
                    r#".jobs[] | select(.name == "{}") | .steps[] | select(.completed_at != null) | "\(.name): \((.completed_at | fromdate) - (.started_at | fromdate))""#,
                    self.job
                ),
            ],
        )
    }
}

impl fmt::Display for WallTimes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "wall time per step of {}, seconds:", self.job)
    }
}

impl Step for WallTimes {
    fn from_arguments(given: Vec<OsString>) -> Result<WallTimes, Failure<'static>> {
        WallTimes::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let invocation = self.invocation();
        let table = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let said = vec![Shown::said(format_args!("{self}\n{table}"))];
        Ok(Telling::reported(said, Ok(Exit::Success)))
    }
}

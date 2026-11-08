mod job;
mod repository;
mod run_id;

use std::ffi::OsString;
use std::fmt;
use std::io::Write;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Invocation, Ports};
use job::Job;
use repository::Repository;
use run_id::RunId;

#[derive(Debug)]
pub struct WallTimes {
    repository: Repository,
    run: RunId,
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
        let mut arguments = Arguments::from(given);
        let repository = &arguments.taken()?;
        let run = &arguments.taken()?;
        let job = &arguments.taken()?;
        arguments.ended()?;
        Ok(WallTimes {
            repository: Repository::new(repository.to_str().ok_or(Failure::Usage)?.to_owned()),
            run: RunId::new(run.to_str().ok_or(Failure::Usage)?.to_owned()),
            job: Job::new(job.to_str().ok_or(Failure::Usage)?.to_owned()),
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let invocation = self.invocation();
        let table = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        writeln!(out, "{self}")?;
        write!(out, "{table}")?;
        Ok(Exit::Success)
    }
}

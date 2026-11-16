use std::error::Error;

use design_jobs::JobState;
use design_model::{JobId, RunId};
use design_runs::{Delivery, NextJob, NextRun, RunState};
use design_store::StreamSchema;
use port_core::Port;
use sqlite_ports::SqliteOpener;
use store_ports::{Argument, Column, IntoArgument, Prepared, Store};

type Sqlite = <SqliteOpener as store_ports::Opens>::Store;

struct Placing;

impl Placing {
    fn job(store: &Port<'_, Sqlite>, state: JobState) -> Result<JobId, Box<dyn Error>> {
        let job = store.read(NextJob)?;
        let arguments = vec![job.raw().into_argument(), state.raw().into_argument()];
        store.write(Prepared::written("INSERT INTO jobs (id, state) VALUES (?1, ?2)", arguments))?;
        Ok(job)
    }

    fn run(store: &Port<'_, Sqlite>, state: RunState) -> Result<RunId, Box<dyn Error>> {
        let job = Placing::job(store, JobState::Open)?;
        let run = store.read(NextRun)?;
        let arguments = vec![run.raw().into_argument(), job.raw().into_argument(), state.raw().into_argument()];
        store.write(Prepared::written(
            "INSERT INTO runs (id, job, pid, deadline, state) VALUES (?1, ?2, 1, 0, ?3)",
            arguments,
        ))?;
        Ok(run)
    }

    fn failure(store: &Port<'_, Sqlite>, delivery: Delivery) -> Result<JobId, Box<dyn Error>> {
        let job = Placing::job(store, JobState::Flailed)?;
        let arguments =
            vec![job.raw().into_argument(), Argument::Text(String::from("told")), delivery.raw().into_argument()];
        store.write(Prepared::written(
            "INSERT INTO failures (job, halted, account, delivery) VALUES (?1, 'flailed', ?2, ?3)",
            arguments,
        ))?;
        Ok(job)
    }
}

laws::lifecycle!(a_job_moves_only_along_its_lifecycle, JobState, Sqlite::in_memory::<StreamSchema>(), Placing::job);

laws::lifecycle!(a_run_moves_only_along_its_lifecycle, RunState, Sqlite::in_memory::<StreamSchema>(), Placing::run);

laws::lifecycle!(
    a_failure_report_moves_only_along_its_lifecycle,
    Delivery,
    Sqlite::in_memory::<StreamSchema>(),
    Placing::failure
);

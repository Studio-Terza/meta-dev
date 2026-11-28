use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use console_ports::Answer;
use pipeline::Verdict;

use crate::failure::Failure;
use crate::limits::{Stage, Tenths};
use crate::ports::{Mark, Ran};

#[derive(Debug)]
pub struct Timed {
    limits: PathBuf,
    stage: String,
    since: Option<PathBuf>,
    program: OsString,
    arguments: Vec<OsString>,
}

impl Timed {
    pub(crate) fn new(
        [limits, stage]: [&OsString; 2],
        since: Option<&OsString>,
        program: &OsString,
        arguments: &[OsString],
    ) -> Timed {
        Timed {
            limits: PathBuf::from(limits),
            stage: stage.to_string_lossy().into_owned(),
            since: since.map(PathBuf::from),
            program: program.clone(),
            arguments: arguments.to_vec(),
        }
    }

    pub(crate) fn answered(&self) -> Result<Answer, Failure> {
        let limit = Stage::read(&self.limits, &self.stage)?;
        let ran = Ran::started(&self.program, &self.arguments)?;
        let spent = Tenths::from(self.since.as_deref().map_or(Ok(ran.spent()), Mark::elapsed)?);
        let verdict = limit.judged(spent)?;
        let (stage, block, status) = (&self.stage, limit.block(), ran.status());
        let failed = (!status.success()).then(|| format!("::error::{stage} failed: {status}\n"));
        let judged = match verdict {
            Verdict::Within => format!("{stage}: within block {block} s, took {spent}\n"),
            Verdict::Over => format!("::error::{stage}: above block {block} s, took {spent}\n"),
        };
        let passed = status.success() && verdict == Verdict::Within;
        let code = if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE };
        Ok(Answer::new(code).showing(failed.unwrap_or_default()).showing(judged))
    }
}

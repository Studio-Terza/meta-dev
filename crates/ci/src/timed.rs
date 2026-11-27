use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::Exit;
use crate::failure::Failure;
use crate::limits::{Limit, Tenths, Verdict};
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

    pub(crate) fn run(&self, out: &mut impl Write) -> Result<Exit, Failure> {
        let limit = Limit::read(&self.limits, &self.stage)?;
        let ran = Ran::started(&self.program, &self.arguments)?;
        let spent = Tenths::from(self.since.as_deref().map_or(Ok(ran.spent()), Mark::elapsed)?);
        let verdict = limit.judged(spent)?;
        let stage = &self.stage;
        let (warn, block) = (limit.warn(), limit.block());
        match verdict {
            Verdict::Within => writeln!(out, "{stage}: within warn {warn} s and block {block} s, took {spent}"),
            Verdict::Warned => {
                writeln!(out, "::warning::{stage}: above warn {warn} s, within block {block} s, took {spent}")
            }
            Verdict::Blocked => writeln!(out, "::error::{stage}: above block {block} s, took {spent}"),
        }?;
        let status = ran.status();
        match (status.success(), verdict) {
            (true, Verdict::Within | Verdict::Warned) => Ok(Exit::Passed),
            (true, Verdict::Blocked) => Ok(Exit::Failed),
            (false, _) => {
                writeln!(out, "::error::{stage} failed: {status}").map(|()| Exit::Failed).map_err(Failure::from)
            }
        }
    }
}

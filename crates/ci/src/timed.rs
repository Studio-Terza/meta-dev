use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use console_ports::Answer;
use dispatch_ports::{Call, DispatchFailure, Ran, Shell};
use file_ports::Files;
use grammar::lines::Lines;
use pipeline::Verdict;
use timer_ports::{Clock, Deadline, Timeout, Wall};

use crate::failure::Failure;
use crate::limits::{Stage, Took};
use crate::ports::Ports;

#[derive(Debug)]
pub struct Timed {
    limits: PathBuf,
    stage: String,
    since: Option<PathBuf>,
    program: String,
    arguments: Vec<String>,
}

impl Timed {
    const CURRENT: &'static str = ".";

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
            program: program.to_string_lossy().into_owned(),
            arguments: arguments.iter().map(|argument| argument.to_string_lossy().into_owned()).collect(),
        }
    }

    pub(crate) fn answered(&self, ports: &Ports<'_, impl Files, impl Clock, impl Shell>) -> Result<Answer, Failure> {
        let limit = Stage::read(ports, &self.limits, &self.stage)?;
        let begun = ports.clock().now();
        let deadline = Deadline::after(begun, Timeout::new(u32::MAX));
        let call = Call::new(self.program.clone(), self.arguments.clone(), PathBuf::from(Timed::CURRENT), deadline);
        let ran = ports.shell().ran(&call.inherited()).map_err(|failure| self.unstarted(failure))?;
        let ended = ports.clock().now();
        let start = self.since.as_deref().map_or(Ok(begun), |mark| Timed::marked(ports, mark, ended))?;
        let spent = Took::from(ended.since(start));
        let verdict = limit.judged(spent)?;
        let (stage, warn, block) = (&self.stage, limit.warn(), limit.block());
        let succeeded = matches!(ran, Ran::Succeeded(_));
        let failed = (!succeeded).then(|| format!("::error::{stage} failed\n"));
        let judged = match verdict {
            Verdict::Within => format!("{stage}: within warn {warn} s and block {block} s, took {spent}\n"),
            Verdict::Warned => {
                format!("::warning::{stage}: above warn {warn} s, within block {block} s, took {spent}\n")
            }
            Verdict::Over => format!("::error::{stage}: above block {block} s, took {spent}\n"),
        };
        let passed = succeeded && verdict != Verdict::Over;
        let code = if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE };
        Ok(Answer::new(code).showing(failed.unwrap_or_default()).showing(judged))
    }

    fn marked(ports: &Ports<'_, impl Files, impl Clock, impl Shell>, mark: &Path, now: Wall) -> Result<Wall, Failure> {
        let text = ports.read(mark)?;
        let first = Lines::from(text.as_str()).into_iter().next().unwrap_or_default();
        let unmarked = || Failure::Unmarked(mark.to_path_buf());
        let seconds = first.parse::<f64>().map_err(|_unparsed| unmarked())?;
        Wall::from_seconds(seconds).filter(|start| *start <= now).ok_or_else(unmarked)
    }

    fn unstarted(&self, failure: DispatchFailure) -> Failure {
        match failure {
            DispatchFailure::Spawn(source) => Failure::Unstarted { program: self.program.clone(), source },
            DispatchFailure::Collect(_) | DispatchFailure::Unjoined | DispatchFailure::Kill(_) => {
                Failure::Dispatched(failure)
            }
        }
    }
}

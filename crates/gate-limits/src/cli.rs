mod contents;
mod document;
mod failure;
mod gate;
mod stage;

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use contents::Contents;
pub use failure::Failure;
pub use gate::Finding;
use gate::Gate;
use gate::Record;
pub use stage::Stage;

#[derive(Debug)]
pub struct Cli {
    limits: PathBuf,
    measured: PathBuf,
    only: Vec<OsString>,
    report_only: Vec<OsString>,
}

impl Cli {
    const DEFAULT_LIMITS: &'static str = "ci/limits.toml";
    const DEFAULT_MEASURED: &'static str = "ci/measured.toml";
    const REPORT_ONLY: &'static str = "--report-only";
    const WHOLE_REACH: &'static str = "--whole-reach";

    pub fn from_args(mut arguments: impl Iterator<Item = OsString>) -> Cli {
        let limits = arguments.next().map_or_else(|| PathBuf::from(Self::DEFAULT_LIMITS), PathBuf::from);
        let measured = arguments.next().map_or_else(|| PathBuf::from(Self::DEFAULT_MEASURED), PathBuf::from);
        let all: Vec<OsString> = arguments.collect();
        let split = all.iter().position(|argument| argument == Self::WHOLE_REACH).unwrap_or(all.len());
        let rest: Vec<OsString> = all.iter().take(split).cloned().collect();
        let reach: Vec<&OsString> = all.iter().skip(split).skip(1).collect();
        let flag = rest.iter().position(|argument| argument == Self::REPORT_ONLY).unwrap_or(rest.len());
        let only = rest.iter().take(flag).cloned().collect();
        let report_only = rest.iter().skip(flag).skip(1).chain(Self::relaxed(&reach)).cloned().collect();
        Cli { limits, measured, only, report_only }
    }

    fn relaxed<'argument>(reach: &[&'argument OsString]) -> Vec<&'argument OsString> {
        let names: &[&OsString] = match reach {
            [reached, _, total, _, names @ ..] if reached == total && *reached != "0" => names,
            _ => &[],
        };
        names.to_vec()
    }

    #[must_use]
    pub fn run(&self) -> ExitCode {
        self.report_to(&mut io::stdout(), &mut io::stderr())
    }

    #[must_use]
    pub fn report_to(&self, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match self.gate() {
            Ok(gate) => {
                write!(out, "{gate}").ok();
                gate.blocking(&self.report_only).map_or(ExitCode::SUCCESS, |_| ExitCode::FAILURE)
            }
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }

    pub fn past_warn(&self) -> Result<Vec<Finding>, Failure> {
        self.gate().map(Gate::past_warn)
    }

    pub fn lower(&self) -> Result<Record, Failure> {
        let record = self.gate()?.lowered()?;
        fs::File::create(&self.limits)
            .and_then(|mut file| write!(file, "{record}"))
            .map(|()| record)
            .map_err(|source| Failure::Io { path: self.limits.clone(), source })
    }

    fn gate(&self) -> Result<Gate, Failure> {
        Gate::judge(Contents::new(&Self::read(&self.limits)?), Contents::new(&Self::read(&self.measured)?), &self.only)
    }

    fn read(path: &Path) -> Result<String, Failure> {
        fs::read_to_string(path).map_err(|source| Failure::Io { path: path.to_path_buf(), source })
    }
}

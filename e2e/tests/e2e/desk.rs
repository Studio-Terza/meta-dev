use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::process::ExitCode;

use ci::Cli;
use tempfile::TempDir;

use crate::fake::Fake;

#[derive(Debug)]
pub enum Scope {
    Workspace,
    Crates(BTreeSet<String>),
}

impl Scope {
    const BUILD: &'static str = "BUILD_SCOPE=";
    const WORKSPACE: &'static str = "--workspace";
    const PACKAGE: &'static str = "-p";

    fn read(report: &str) -> Scope {
        let value = |key: &str| report.lines().find_map(|line| line.strip_prefix(key)).unwrap_or_default().to_owned();
        let build = value(Self::BUILD);
        if build == Self::WORKSPACE {
            Scope::Workspace
        } else {
            Scope::Crates(build.split_whitespace().filter(|word| *word != Self::PACKAGE).map(str::to_owned).collect())
        }
    }

    pub fn built(&self, everything: impl Iterator<Item = String>) -> BTreeSet<String> {
        match self {
            Scope::Workspace => everything.collect(),
            Scope::Crates(built) => built.clone(),
        }
    }
}

#[derive(Debug)]
struct Answer {
    exit: ExitCode,
    out: String,
}

#[derive(Debug)]
pub struct Desk {
    scratch: TempDir,
}

impl Desk {
    pub fn new() -> Result<Desk, Box<dyn Error>> {
        Ok(Desk { scratch: TempDir::new()? })
    }

    pub fn scope(&self, fake: &Fake, event: &str, base: &str) -> Result<Scope, Box<dyn Error>> {
        let answer = self.answer(
            fake,
            &["reach", &self.file("metadata.json"), event, base, &self.file("env"), &self.file("summary")],
        );
        if answer.exit == ExitCode::SUCCESS { Ok(Scope::read(&answer.out)) } else { Err(answer.out.into()) }
    }

    fn answer(&self, fake: &Fake, arguments: &[&str]) -> Answer {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::answer(arguments.iter().map(OsString::from), fake, &mut out, &mut err);
        out.extend(err);
        Answer { exit, out: String::from_utf8_lossy(&out).into_owned() }
    }

    fn file(&self, name: &str) -> String {
        self.scratch.path().join(name).display().to_string()
    }
}

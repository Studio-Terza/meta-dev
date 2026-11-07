use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use ci::Cli;

use crate::fake::Fake;

#[derive(Debug)]
pub enum Build {
    Workspace,
    Crates(BTreeSet<String>),
}

#[derive(Debug)]
pub struct Scope {
    build: Build,
    mutated: BTreeSet<String>,
}

impl Scope {
    const BUILD: &'static str = "BUILD_SCOPE=";
    const WORKSPACE: &'static str = "--workspace";
    const PACKAGE: &'static str = "-p";
    const MUTANTS: &'static str = "MUTANTS_SCOPE=";
    const TEST_PACKAGE: &'static str = "--test-package=";

    fn read(report: &str) -> Self {
        let value = |key: &str| report.lines().find_map(|line| line.strip_prefix(key)).unwrap_or_default().to_owned();
        let build = value(Self::BUILD);
        Self {
            build: if build == Self::WORKSPACE {
                Build::Workspace
            } else {
                {
                    Build::Crates(
                        build.split_whitespace().filter(|word| *word != Self::PACKAGE).map(str::to_owned).collect(),
                    )
                }
            },
            mutated: value(Self::MUTANTS)
                .split_whitespace()
                .filter_map(|word| word.strip_prefix(Self::TEST_PACKAGE))
                .map(str::to_owned)
                .collect(),
        }
    }

    pub fn built(&self, everything: impl Iterator<Item = String>) -> BTreeSet<String> {
        match &self.build {
            Build::Workspace => everything.collect(),
            Build::Crates(built) => built.clone(),
        }
    }

    pub fn mutated(&self) -> &BTreeSet<String> {
        &self.mutated
    }
}

#[derive(Debug)]
struct Answer {
    exit: ExitCode,
    out: String,
}

#[derive(Debug, Default)]
pub struct Desk {
    root: PathBuf,
}

impl Desk {
    pub fn scope(&self, fake: &Fake, event: &str, base: &str) -> Result<Scope, Box<dyn Error>> {
        let answer = self.answer(
            fake,
            &["reach", &self.file("metadata.json"), event, base, &self.file("env"), &self.file("summary")],
        );
        (answer.exit == ExitCode::SUCCESS).then(|| Scope::read(&answer.out)).ok_or_else(|| answer.out.clone().into())
    }

    pub fn labels_pass(&self, fake: &Fake, base: &str) -> bool {
        self.answer(fake, &["labels", base, "1", &self.file("labels.json")]).exit == ExitCode::SUCCESS
    }

    fn answer(&self, fake: &Fake, arguments: &[&str]) -> Answer {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::answer(arguments.iter().map(OsString::from), fake, &mut out, &mut err);
        out.extend(err);
        Answer { exit, out: String::from_utf8_lossy(&out).into_owned() }
    }

    fn file(&self, name: &str) -> String {
        self.root.join(name).display().to_string()
    }
}

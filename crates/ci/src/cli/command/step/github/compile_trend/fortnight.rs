use std::fmt;
use std::path::{Path, PathBuf};

use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use super::build::Build;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Deserialize)]
#[serde(transparent)]
struct RunId(u64);

#[derive(Clone, Copy, Debug)]
enum Stamp {
    Created,
    Merged,
}

impl fmt::Display for Stamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self {
            Stamp::Created => "createdAt",
            Stamp::Merged => "mergedAt",
        };
        write!(formatter, "(.{field} | fromdate) > now - {} * {}", Fortnight::DAYS, Fortnight::DAY_SECONDS)
    }
}

#[derive(Debug)]
pub struct Fortnight {
    downloads: PathBuf,
}

impl Fortnight {
    const DAYS: u32 = 14;
    const DAY_SECONDS: u32 = 86_400;
    const SEEN: &'static str = "100";
    const ARTIFACT: &'static str = "cold-build";
    const WORKFLOW: &'static str = "cold-build.yml";
    const BRANCH: &'static str = "main";

    pub(super) fn downloading_to(downloads: PathBuf) -> Fortnight {
        Fortnight { downloads }
    }

    pub(super) fn downloaded(&self, ports: &impl Ports) -> Result<Vec<Build>, Failure<'static>> {
        let invocation = Invocation::new(
            "gh",
            [
                "run".to_owned(),
                "list".to_owned(),
                "--workflow".to_owned(),
                Self::WORKFLOW.to_owned(),
                "--branch".to_owned(),
                Self::BRANCH.to_owned(),
                "--status".to_owned(),
                "success".to_owned(),
                "--limit".to_owned(),
                Self::SEEN.to_owned(),
                "--json".to_owned(),
                "databaseId,createdAt".to_owned(),
                "--jq".to_owned(),
                format!("[.[] | select({}) | .databaseId]", Stamp::Created),
            ],
        );
        let listed = ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        let runs: Vec<RunId> = serde_json::from_str(listed.as_ref()).map_err(Failure::Runs)?;
        let directories: Vec<PathBuf> = runs.iter().map(|run| self.downloads.join(run.to_string())).collect();
        let invocations: Vec<Invocation> =
            runs.iter().zip(&directories).map(|(run, directory)| Self::download(*run, directory)).collect();
        Failure::started(ports.outputs(&invocations), &invocations)?;
        Build::measured_in_all(ports.files(), &directories)
    }

    fn download(run: RunId, directory: &Path) -> Invocation {
        Invocation::new(
            "gh",
            [
                "run".into(),
                "download".into(),
                run.to_string().into(),
                "--name".into(),
                Self::ARTIFACT.into(),
                "--dir".into(),
                directory.as_os_str().to_owned(),
            ],
        )
    }

    pub(super) fn merged(ports: &impl Ports) -> Result<Text, Failure<'static>> {
        let invocation = Invocation::new(
            "gh",
            [
                "pr".to_owned(),
                "list".to_owned(),
                "--state".to_owned(),
                "merged".to_owned(),
                "--base".to_owned(),
                Self::BRANCH.to_owned(),
                "--search".to_owned(),
                "sort:updated-desc".to_owned(),
                "--limit".to_owned(),
                Self::SEEN.to_owned(),
                "--json".to_owned(),
                "number,title,mergedAt".to_owned(),
                "--jq".to_owned(),
                format!(r##".[] | select({}) | "#\(.number) \(.title)""##, Stamp::Merged),
            ],
        );
        ports.output(&invocation).map_err(Failure::starting(&invocation))
    }
}

impl fmt::Display for Fortnight {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "the previous {} days", Self::DAYS)
    }
}

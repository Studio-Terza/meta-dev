use std::fmt;
use std::path::PathBuf;

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

    pub(super) fn downloaded(&self, ports: &impl Ports) -> Result<Vec<Build>, Failure> {
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
        runs.into_iter().map(|run| self.download(run, ports)).collect()
    }

    fn download(&self, run: RunId, ports: &impl Ports) -> Result<Build, Failure> {
        let directory = self.downloads.join(run.to_string());
        let invocation = Invocation::new(
            "gh",
            [
                "run".into(),
                "download".into(),
                run.to_string().into(),
                "--name".into(),
                Self::ARTIFACT.into(),
                "--dir".into(),
                directory.clone().into_os_string(),
            ],
        );
        ports.output(&invocation).map_err(Failure::starting(&invocation))?;
        Build::try_from(directory.as_path())
    }

    pub(super) fn merged(ports: &impl Ports) -> Result<Text, Failure> {
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

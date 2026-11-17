use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use text_lines::OneLine;

use super::super::kept::Commits;
use crate::cli::ports::{Invocation, Text};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Run {
    database_id: u64,
    head_sha: OneLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
struct Uploaded {
    index: u16,
    count: u16,
}

#[derive(Debug)]
pub(super) struct Complete {
    run: Run,
    shards: u16,
}

#[derive(Debug)]
pub(super) enum Found {
    None,
    Unreadable(String),
    Reused { run: u64, tested: Text, head: String },
}

impl Run {
    const WORKFLOW: &'static str = "ci.yml";
    const SEEN: &'static str = "10";
    const UPLOADED: &'static str = "[.artifacts[] | select(.expired | not) | .name | capture(\"^mutants-result-(?<index>[0-9]+)-of-(?<count>[0-9]+)$\") | {index: (.index | tonumber), count: (.count | tonumber)}]";

    pub(super) fn listing(repository: &OsStr, branch: &OsStr) -> Invocation {
        let fixed = ["run", "list", "--workflow", Self::WORKFLOW, "--event", "pull_request", "--status", "completed"];
        let named = [
            OsStr::new("--branch"),
            branch,
            OsStr::new("--repo"),
            repository,
            OsStr::new("--limit"),
            OsStr::new(Self::SEEN),
        ];
        let fields = ["--json", "databaseId,headSha"];
        Invocation::new(
            "gh",
            fixed
                .iter()
                .map(OsString::from)
                .chain(named.iter().map(OsString::from))
                .chain(fields.iter().map(OsString::from)),
        )
    }

    pub(super) fn listed(text: &Text) -> Result<Vec<Run>, serde_json::Error> {
        serde_json::from_str(text.as_ref())
    }

    pub(super) fn ancestry(&self, head: &Text) -> Invocation {
        Invocation::new("git", ["merge-base", "--is-ancestor", self.head_sha.as_ref(), head.as_ref()])
    }

    pub(super) fn uploads(&self, repository: &OsStr) -> Invocation {
        let path =
            format!("repos/{}/actions/runs/{}/artifacts?per_page=100", repository.to_string_lossy(), self.database_id);
        Invocation::new("gh", ["api", path.as_str(), "--jq", Self::UPLOADED])
    }

    pub(super) fn complete(self, uploaded: &Text) -> Result<Option<Complete>, serde_json::Error> {
        let listed: BTreeSet<Uploaded> = serde_json::from_str(uploaded.as_ref())?;
        let shards = listed.first().map_or(0, |first| first.count);
        let expected: BTreeSet<Uploaded> = (0..shards).map(|index| Uploaded { index, count: shards }).collect();
        Ok(NonZeroU16::new(shards).and_then(|_| (listed == expected).then_some(Complete { run: self, shards })))
    }
}

impl Complete {
    const DOWNLOADED: &'static str = "download";
    const TESTED: &'static str = "tested";

    pub(super) fn downloads(&self, repository: &OsStr, directory: &Path) -> Vec<Invocation> {
        (0..self.shards)
            .map(|index| {
                let name = format!("mutants-result-{index}-of-{}", self.shards);
                let into = directory.join(Self::DOWNLOADED).join(index.to_string());
                let run = self.run.database_id.to_string();
                Invocation::new(
                    "gh",
                    [OsStr::new("run"), OsStr::new("download"), OsStr::new(&run), OsStr::new("--repo"), repository]
                        .into_iter()
                        .chain([OsStr::new("--name"), OsStr::new(&name), OsStr::new("--dir"), into.as_os_str()])
                        .map(OsString::from)
                        .collect::<Vec<OsString>>(),
                )
            })
            .collect()
    }

    pub(super) fn tested(&self, directory: &Path) -> Vec<PathBuf> {
        (0..self.shards)
            .map(|index| directory.join(Self::DOWNLOADED).join(index.to_string()).join(Self::TESTED))
            .collect()
    }

    pub(super) fn found(&self, tested: &[Text]) -> Found {
        let read: BTreeSet<Option<Commits>> = tested.iter().map(Commits::read).collect();
        let agreed = read.first().cloned().flatten().filter(|_| read.len() == 1);
        agreed.filter(|found| found.head().as_ref() == self.run.head_sha.as_ref()).map_or_else(
            || Found::Unreadable(String::from("its shards did not all record the head it tested")),
            |_| Found::Reused {
                run: self.run.database_id,
                tested: tested.first().cloned().unwrap_or_default(),
                head: self.run.head_sha.to_string(),
            },
        )
    }
}

impl Found {
    pub(super) fn failed(failure: &impl fmt::Display) -> Found {
        Found::Unreadable(failure.to_string())
    }

    pub(super) fn tested(&self) -> Option<&Text> {
        match self {
            Found::Reused { tested, .. } => Some(tested),
            Found::None | Found::Unreadable(_) => None,
        }
    }
}

impl fmt::Display for Found {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Found::None => writeln!(
                formatter,
                "mutants: no earlier run on an ancestor of this head kept every shard's result, so this run mutates every changed line"
            ),
            Found::Unreadable(why) => {
                writeln!(
                    formatter,
                    "mutants: the earlier runs cannot be reused ({why}), so this run mutates every changed line"
                )
            }
            Found::Reused { run, head, .. } => writeln!(formatter, "mutants: reusing run {run}, which tested {head}"),
        }
    }
}

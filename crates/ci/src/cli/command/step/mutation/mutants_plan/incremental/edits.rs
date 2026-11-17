use std::path::Path;

use file_ports::Files;
use grammar_hunk::Patch;

use super::super::super::kept::Commits;
use super::boundary::{Metadata, Reason, TestCode};
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Invocation, Ports, Text};

#[derive(Debug)]
pub(super) struct Edits<'run> {
    metadata: &'run Path,
    commits: &'run Commits,
}

impl<'run> Edits<'run> {
    const CFG_TEST: &'static str = "#[cfg(test)]";

    pub(super) fn new(metadata: &'run Path, commits: &'run Commits) -> Edits<'run> {
        Edits { metadata, commits }
    }

    pub(super) fn found(&self, ports: &impl Ports, head: &Text) -> Result<Option<Reason>, Failure<'static>> {
        let range = [self.commits.head().as_ref(), "..", head.as_ref()].concat();
        let log = Invocation::new(
            "git",
            ["log", "--first-parent", "--no-merges", "--format=", "-p", "-U0", "--no-renames", range.as_str()],
        );
        let logged = ports.output(&log).map_err(Failure::starting(&log))?;
        let grepped = ports
            .output(&Invocation::new(
                "git",
                ["grep", "-l", "-F", Self::CFG_TEST, self.commits.head().as_ref(), "--", "*.rs"],
            ))
            .unwrap_or_default();
        let path = self.metadata.to_path_buf();
        let text = ports.files().read_text(&path).map_err(|failure| Failure::unusable(path, failure))?;
        let metadata = Metadata::read(&text).map_err(Failure::Metadata)?;
        let tests = TestCode::new(&metadata, &grepped, self.commits);
        Ok(logged.as_ref().parse::<Patch>().ok().map_or(Some(Reason::Unreadable), |patch| tests.edited(&patch)))
    }
}
